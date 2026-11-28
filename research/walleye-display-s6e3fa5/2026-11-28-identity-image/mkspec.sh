#!/bin/sh
# mkspec.sh CPIO ROOT_OF_PLAIN_FILES OLDV NEWMODROOT NEWV : gen_init_cpio spec of the cpio, its modules read from NEWMODROOT/lib/modules/NEWV
set -eu
cpio -tv --quiet < "$1" | awk -v P="$2" -v OV="$3" -v M="$4" -v NV="$5" '
{ perm=$1; name=$(NF); if ($(NF-1)=="->") { name=$(NF-2); tgt=$NF } 
  mode=0; s=substr(perm,2,9); for(i=0;i<9;i++){ c=substr(s,i+1,1); if(c!="-") mode+=2^(8-i) }
  m=sprintf("%o", mode); t=substr(perm,1,1)
  n=name; sub(OV, NV, n)
  if (t=="d") printf "dir /%s %s 0 0\n", n, m
  else if (t=="c") { printf "nod /%s %s 0 0 c %s\n", n, m, (name=="dev/console" ? "5 1" : "1 3") }
  else if (t=="l") printf "slink /%s %s %s 0 0\n", n, tgt, m
  else if (index(name,"lib/modules/")==1) printf "file /%s %s/%s %s 0 0\n", n, M, n, m
  else printf "file /%s %s/%s %s 0 0\n", n, P, name, m }'
