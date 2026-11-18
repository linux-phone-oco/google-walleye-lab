# dts-norm.awk: one line per property of a decompiled dtb ("dtc -I dtb -O dts -s"): "PATH PROP = VALUE", phandle cells resolved to node paths, so two dtbs whose phandle numbers differ compare by what they mean. Run twice over the same file: `gawk -f dts-norm.awk X.dts X.dts | LC_ALL=C sort`.
# Resolved: clocks, power-domains, iommus, interconnects, resets, dmas, phys, mboxes, thermal-sensors, io-channels, smem-states, gpio lists, supplies, pinctrl-N, nvmem-cells, gpio-ranges, halt-regs, tcsr-reg, freq-domain, monitored-battery. Unresolved ones stay raw ("<RAW ...>" or "?0x..") and are named by the caller.
function pathnow(  i,p){p="";for(i=1;i<=d;i++)p=p "/" st[i];return p==""?"/":p}
function h2n(x){return strtonum(x)}
function ref(x,  n){n=h2n(x); return (n in pp)?"&" pp[n]:"?" x}
# cells(name) -> provider cells property for list-of-(phandle,args) properties
function provcell(name){
  if(name=="clocks"||name=="assigned-clocks")return "#clock-cells"
  if(name=="assigned-clock-parents")return "#clock-cells"
  if(name=="nvmem-cells")return "#nvmem-cell-cells0"
  if(name=="gpio-ranges")return "#X3"
  if(name=="qcom,halt-regs")return "#X3"
  if(name=="qcom,tcsr-reg")return "#X1"
  if(name=="qcom,freq-domain")return "#X1"
  if(name=="monitored-battery")return "#X0"
  if(name=="power-domains")return "#power-domain-cells"
  if(name=="iommus")return "#iommu-cells"
  if(name=="interconnects")return "#interconnect-cells"
  if(name=="resets")return "#reset-cells"
  if(name=="dmas")return "#dma-cells"
  if(name=="phys")return "#phy-cells"
  if(name=="mboxes")return "#mbox-cells"
  if(name=="thermal-sensors")return "#thermal-sensor-cells"
  if(name=="io-channels")return "#io-channel-cells"
  if(name=="qcom,smem-states")return "#qcom,smem-state-cells"
  if(name=="hwlocks")return "#hwlock-cells"
  if(name=="pwms")return "#pwm-cells"
  if(name=="interrupts-extended")return "#interrupt-cells"
  if(name ~ /(^|-)gpios?$/ )return "#gpio-cells"
  return ""
}
function single(name){return name ~ /-supply$/ || name ~ /^pinctrl-[0-9]+$/ || name=="interrupt-parent"||name=="memory-region"||name=="next-level-cache"||name=="cpu-idle-states"||name=="cpu"||name=="msi-parent"||name=="operating-points-v2"||name=="required-opps"||name=="sleep-parent"||name=="trip"||name=="remote-endpoint"||name=="qcom,freq-domain"||name=="power-domain"||name=="vdd-supply"||name=="iommu-map"||name=="mmu-masters"||name=="interrupt-affinity"||name=="phandle-ref"||name=="qcom,rpmh-regulator-ids"}
BEGIN{FS="\n"}
FNR==1{pass++; d=0}
{
  line=$0
  if(line ~ /\{$/ && line !~ /=/){ s=line; sub(/^[ \t]+/,"",s); sub(/ *\{$/,"",s); if(s=="/")s=""; st[++d]=s; next }
  if(line ~ /^[ \t]*\};$/){d--;next}
  if(line !~ /;$/ ) next
  p=pathnow(); s=line; sub(/^[ \t]+/,"",s); sub(/;$/,"",s)
  name=s; sub(/ *=.*/,"",name); val=""; if(s ~ /=/){val=s; sub(/^[^=]*= */,"",val)}
  if(pass==1){
    if(name=="phandle"){v=val;gsub(/[<>]/,"",v); pp[h2n(v)]=p}
    if(name ~ /^#.*-cells$/){v=val;gsub(/[<>]/,"",v); cells[p SUBSEP name]=h2n(v)}
    next
  }
  if(p ~ /^\/\/(__symbols__|__fixups__|__local_fixups__|__overrides__)/) next
  if(name=="phandle") next
  out=val
  if(val ~ /^<.*>$/){
    v=val; gsub(/[<>]/,"",v); n=split(v,a," ")
    pc=provcell(name)
    if(single(name) && !pc){ o="";for(i=1;i<=n;i++)o=o (i>1?" ":"") ref(a[i]); out="<" o ">" }
    else if(pc!=""){
      o=""; i=1; ok=1
      while(i<=n){
        r=ref(a[i]); 
        if(r ~ /^\?/){ok=0;break}
        tp=pp[h2n(a[i])]; c=(pc ~ /^#X[0-9]$/)?substr(pc,3)+0:((pc=="#nvmem-cell-cells0")?0:(((tp SUBSEP pc) in cells)?cells[tp SUBSEP pc]:-1))
        if(c<0){ok=0;break}
        o=o (o==""?"":" ") r; for(j=1;j<=c;j++)o=o " " a[i+j]; i+=c+1
      }
      if(ok) out="<" o ">"; else out="<RAW " v ">"
    }
  }
  print p " " name " = " out
}
