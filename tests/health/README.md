# This phone's own health checks

The generic checks come with devlab (linux-phone-oco/lab-tools, `health/suite/checks/`): storage, kernel,
signal processors, sound cards, Wi-Fi, modem, Bluetooth, display and input, the other parts, the shell's
sound. Put here only what is this phone's own - its amplifier, its quirk - as small POSIX shell files:

    base/NAME.sh    the hardware and the system, no graphical shell in the way
    shell/NAME.sh   what a graphical shell adds on top

Each check sources the suite's words and prints a line per test, `pass|fail|skip TEST COMPONENT detail`
(COMPONENT: a linux-phone-oco/docs component id). A skip says what could not be measured and why.

```sh
# the speaker amplifier answers on its bus
. /usr/lib/devlab/health/lib.sh
check amp-present audio-speaker "$(i2cget -f -y 9 0x4c 0x00 >/dev/null 2>&1 && echo true)" "amplifier at 9-004c"
```

`devlab suite put now --runner FILE` carries them to the phone; `devlab suite run` runs everything now.
