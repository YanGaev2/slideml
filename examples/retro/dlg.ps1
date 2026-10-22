# Lists PowerPoint modal dialogs (#32770) with their text. With -Dismiss, presses the dialog button.
param([switch]$Dismiss)
Add-Type @'
using System; using System.Text; using System.Collections.Generic; using System.Runtime.InteropServices;
public class PptDlg { public delegate bool P(IntPtr h, IntPtr l);
[DllImport("user32.dll")] static extern bool EnumWindows(P p, IntPtr l);
[DllImport("user32.dll")] static extern bool EnumChildWindows(IntPtr w, P p, IntPtr l);
[DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
[DllImport("user32.dll")] static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
[DllImport("user32.dll")] static extern int GetClassName(IntPtr h, StringBuilder s, int n);
[DllImport("user32.dll")] public static extern IntPtr SendMessage(IntPtr h, uint m, IntPtr w, IntPtr l);
static string Txt(IntPtr h){var t=new StringBuilder(1024);GetWindowText(h,t,1024);return t.ToString();}
static string Cls(IntPtr h){var c=new StringBuilder(256);GetClassName(h,c,256);return c.ToString();}
public static List<string> Run(uint pid, bool dismiss) { var r = new List<string>();
 EnumWindows((h,l)=>{ uint p; GetWindowThreadProcessId(h,out p);
  if(p==pid && Cls(h)=="#32770"){ var line="dialog "+h+" '"+Txt(h)+"':"; IntPtr btn=IntPtr.Zero;
   EnumChildWindows(h,(c,l2)=>{ var cl=Cls(c); var t=Txt(c); if(t.Length>0) line+=" ["+cl+":"+t+"]"; if(cl=="Button"&&btn==IntPtr.Zero) btn=c; return true;},IntPtr.Zero);
   if(dismiss && btn!=IntPtr.Zero){ SendMessage(btn,0x00F5,IntPtr.Zero,IntPtr.Zero); line+=" -> pressed"; }
   r.Add(line);} return true;}, IntPtr.Zero); return r; } }
'@
Get-Date -Format T
foreach ($p in Get-Process POWERPNT -ErrorAction SilentlyContinue) { [PptDlg]::Run([uint32]$p.Id, [bool]$Dismiss) }
Get-CimInstance Win32_Process -Filter "Name='powershell.exe'" | Where-Object { $_.CommandLine -like '*render.ps1*' } |
  ForEach-Object { "render pid={0} since {1}: {2}" -f $_.ProcessId, $_.CreationDate.ToString('T'), ($_.CommandLine -replace '.*-Pptx\s+','') }
