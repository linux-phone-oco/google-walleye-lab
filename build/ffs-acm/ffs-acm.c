/* ffs-acm: a serial console for the walleye phone through FunctionFS (decision 2026-11-17-walleye-ffs-acm-single-interface,
 * research walleye-ffs-acm-helper-how-it-stands). One interface class 2/2/1 with interrupt-in, bulk-out, bulk-in: no CDC functional
 * descriptors (f_fs refuses them), cdc_acm on the host takes it as a single combined interface and gives /dev/ttyACM*.
 * Order matters: descriptors then strings go to ep0 first; only then the UDC name is written (before that the bind is -ENODEV).
 * No function attribute is ever written (the 5b panic path). A pty shell is bridged to the bulk endpoints.
 * Static aarch64, libc only, no threads, no arguments (every path is a constant below). Based on the idea of the kernel's
 * tools/usb/ffs-test.c (GPL-2.0); this file is GPL-2.0 too.
 * Needs in the ramdisk: /dev/ffs (dir), /dev/ptmx and /dev/pts mounted, /bin/sh, /sys/kernel/config with gadget g1 and functions/ffs.acm.
 */
#define _GNU_SOURCE
#include <dirent.h>
#include <errno.h>
#include <fcntl.h>
#include <poll.h>
#include <signal.h>
#include <stdarg.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/ioctl.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <unistd.h>

#define FFS_DIR "/dev/ffs"
#define UDC_LIST "/sys/class/udc"
#define UDC_FILE "/sys/kernel/config/usb_gadget/g1/UDC"

#define FFS_DESC_MAGIC_V2 3u
#define FFS_STR_MAGIC 2u
#define FFS_HAS_FS 1u
#define FFS_HAS_HS 2u
#define FFS_HAS_SS 4u

enum { EV_BIND, EV_UNBIND, EV_ENABLE, EV_DISABLE, EV_SETUP, EV_SUSPEND, EV_RESUME };

struct ev { uint8_t bRequestType, bRequest; uint16_t wValue, wIndex, wLength; uint8_t type, pad[3]; } __attribute__((packed));

static int kmsg = -1;
static void say(const char *fmt, ...)
{
	char b[256], c[300];
	va_list ap;
	va_start(ap, fmt);
	vsnprintf(b, sizeof b, fmt, ap);
	va_end(ap);
	int n = snprintf(c, sizeof c, "walleye-ffs-acm: %s\n", b);
	if (kmsg >= 0 && write(kmsg, c, n) < 0) { /* nothing more to do */ }
}

/* ---- the blobs, built into a buffer with little-endian puts ---- */
static unsigned char blob[512];
static size_t bl;
static void p8(uint8_t v) { blob[bl++] = v; }
static void p16(uint16_t v) { p8(v & 255); p8(v >> 8); }
static void p32(uint32_t v) { p16(v & 0xffff); p16(v >> 16); }

static void iface(void) { p8(9); p8(4); p8(0); p8(0); p8(3); p8(2); p8(2); p8(1); p8(1); }
static void ep(uint8_t addr, uint8_t attr, uint16_t max, uint8_t interval) { p8(7); p8(5); p8(addr); p8(attr); p16(max); p8(interval); }
static void comp(uint16_t per) { p8(6); p8(0x30); p8(0); p8(0); p16(per); }

static size_t build_descriptors(void)
{
	bl = 0;
	p32(FFS_DESC_MAGIC_V2); p32(0); /* length patched below */
	p32(FFS_HAS_FS | FFS_HAS_HS | FFS_HAS_SS);
	p32(4); p32(4); p32(7); /* descriptor counts: fs, hs, ss */
	iface(); ep(0x81, 3, 10, 9); ep(0x02, 2, 64, 0); ep(0x83, 2, 64, 0);        /* full speed */
	iface(); ep(0x81, 3, 10, 9); ep(0x02, 2, 512, 0); ep(0x83, 2, 512, 0);      /* high speed */
	iface(); ep(0x81, 3, 10, 9); comp(10); ep(0x02, 2, 1024, 0); comp(0); ep(0x83, 2, 1024, 0); comp(0); /* super speed */
	uint32_t len = bl;
	blob[4] = len & 255; blob[5] = (len >> 8) & 255; blob[6] = (len >> 16) & 255; blob[7] = len >> 24;
	return bl;
}

static size_t build_strings(void)
{
	static const char s[] = "walleye console";
	bl = 0;
	p32(FFS_STR_MAGIC); p32(0); p32(1); p32(1); /* length, one string, one language */
	p16(0x0409);
	for (size_t i = 0; i <= strlen(s); i++) p8(s[i]);
	uint32_t len = bl;
	blob[4] = len & 255; blob[5] = (len >> 8) & 255; blob[6] = (len >> 16) & 255; blob[7] = len >> 24;
	return bl;
}

/* ---- the shell on a pty ---- */
static int master = -1;
static pid_t shell = -1, rd = -1, wr = -1;

static int start_shell(void)
{
	master = posix_openpt(O_RDWR | O_NOCTTY);
	if (master < 0 || grantpt(master) < 0 || unlockpt(master) < 0) { say("pty failed errno %d", errno); return -1; }
	char *name = ptsname(master);
	if (!name) { say("ptsname failed errno %d", errno); return -1; }
	shell = fork();
	if (shell == 0) {
		setsid();
		int s = open(name, O_RDWR);
		if (s < 0) _exit(127);
		ioctl(s, TIOCSCTTY, 0);
		dup2(s, 0); dup2(s, 1); dup2(s, 2);
		if (s > 2) close(s);
		close(master);
		setenv("PATH", "/bin:/sbin:/usr/bin:/usr/sbin", 1);
		setenv("TERM", "dumb", 1);
		execl("/bin/sh", "sh", (char *)0);
		_exit(127);
	}
	say("shell pid %d on %s", (int)shell, name);
	return shell < 0 ? -1 : 0;
}

/* one forwarding child per direction: a blocked bulk-in write (no host reading) never blocks the other direction */
static pid_t pump(int from, int to)
{
	pid_t p = fork();
	if (p != 0) return p;
	char b[1024];
	for (;;) {
		ssize_t n = read(from, b, sizeof b);
		if (n < 0 && errno == EINTR) continue;
		if (n <= 0) _exit(0);
		for (ssize_t o = 0; o < n;) {
			ssize_t w = write(to, b + o, n - o);
			if (w < 0 && errno == EINTR) continue;
			if (w < 0) _exit(0);
			o += w;
		}
	}
}

static int e1 = -1, e2 = -1, e3 = -1;
static void stop_bridge(void)
{
	if (rd > 0) kill(rd, SIGKILL);
	if (wr > 0) kill(wr, SIGKILL);
	while (waitpid(-1, 0, WNOHANG) > 0) { }
	rd = wr = -1;
	if (e1 >= 0) close(e1);
	if (e2 >= 0) close(e2);
	if (e3 >= 0) close(e3);
	e1 = e2 = e3 = -1;
}

static void start_bridge(void)
{
	stop_bridge();
	e1 = open(FFS_DIR "/ep1", O_WRONLY);
	e2 = open(FFS_DIR "/ep2", O_RDONLY);
	e3 = open(FFS_DIR "/ep3", O_WRONLY);
	if (e1 < 0 || e2 < 0 || e3 < 0) { say("opening ep1..ep3 failed errno %d", errno); stop_bridge(); return; }
	rd = pump(e2, master);
	wr = pump(master, e3);
	say("bridge up (host to pty pid %d, pty to host pid %d)", (int)rd, (int)wr);
}

static void setup(int ep0, const struct ev *e)
{
	unsigned char b[64];
	int in = e->bRequestType & 0x80;
	if ((e->bRequestType & 0x7f) == 0x21 && e->bRequest == 0x20 && !in) { /* SET_LINE_CODING: take the data */
		if (read(ep0, b, e->wLength > sizeof b ? sizeof b : e->wLength) < 0) say("SET_LINE_CODING errno %d", errno);
	} else if (e->bRequestType == 0xa1 && e->bRequest == 0x21) { /* GET_LINE_CODING: 115200 8N1 */
		static const unsigned char lc[7] = { 0x00, 0xc2, 0x01, 0x00, 0, 0, 8 };
		if (write(ep0, lc, e->wLength < 7 ? e->wLength : 7) < 0) say("GET_LINE_CODING errno %d", errno);
	} else if (e->bRequestType == 0x21 && e->bRequest == 0x22) { /* SET_CONTROL_LINE_STATE: ack */
		if (read(ep0, b, 0) < 0) say("SET_CONTROL_LINE_STATE errno %d", errno);
	} else { /* anything else: stall by the wrong-direction call */
		say("setup %02x %02x stalled", e->bRequestType, e->bRequest);
		if (in) { if (read(ep0, b, 0) < 0) { } } else { if (write(ep0, b, 0) < 0) { } }
	}
}

int main(void)
{
	kmsg = open("/dev/kmsg", O_WRONLY);
	signal(SIGPIPE, SIG_IGN);
	say("start");
	int ep0 = open(FFS_DIR "/ep0", O_RDWR);
	if (ep0 < 0) { say("open ep0 failed errno %d", errno); return 1; }
	size_t n = build_descriptors();
	if (write(ep0, blob, n) != (ssize_t)n) { say("descriptors write failed errno %d", errno); return 1; }
	n = build_strings();
	if (write(ep0, blob, n) != (ssize_t)n) { say("strings write failed errno %d", errno); return 1; }
	say("descriptors and strings written");
	if (start_shell() < 0) return 1;
	/* now the bind */
	char udc[256] = "";
	DIR *dd = opendir(UDC_LIST);
	struct dirent *de;
	while (dd && (de = readdir(dd))) if (de->d_name[0] != '.') { snprintf(udc, sizeof udc, "%s", de->d_name); break; }
	if (dd) closedir(dd);
	udc[strcspn(udc, "\n")] = 0;
	if (!udc[0]) { say("NO UDC"); return 1; }
	say("about to bind %s", udc);
	int u = open(UDC_FILE, O_WRONLY);
	if (u < 0 || write(u, udc, strlen(udc)) < 0) { say("UDC bind failed errno %d", errno); return 1; }
	close(u);
	say("gadget bound to %s", udc);
	for (;;) {
		struct ev e[4];
		ssize_t r = read(ep0, e, sizeof e);
		if (r < 0 && errno == EINTR) continue;
		if (r < 0) { say("ep0 read errno %d", errno); break; }
		for (size_t i = 0; i < (size_t)r / sizeof e[0]; i++) {
			switch (e[i].type) {
			case EV_ENABLE: say("enable"); start_bridge(); break;
			case EV_DISABLE: say("disable"); stop_bridge(); break;
			case EV_UNBIND: say("unbind"); stop_bridge(); break;
			case EV_BIND: say("bind"); break;
			case EV_SETUP: setup(ep0, &e[i]); break;
			default: break;
			}
		}
	}
	return 1;
}
