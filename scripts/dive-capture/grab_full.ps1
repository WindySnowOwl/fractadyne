param([int]$ProcId, [string]$OutDir, [double]$Seconds = 30, [int]$IntervalMs = 60)
# Capture one process's main window repeatedly via PrintWindow (no input is sent anywhere).
# Windows PowerShell 5.1 (System.Drawing). Frames: <OutDir>\f<ms since start>.jpg, 480 px wide.
Add-Type -AssemblyName System.Drawing
Add-Type @"
using System;
using System.Runtime.InteropServices;
using System.Text;
public static class W {
  public delegate bool EnumProc(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] public static extern bool EnumWindows(EnumProc f, IntPtr l);
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool ClientToScreen(IntPtr h, ref POINT p);
  [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr h, IntPtr hdc, uint f);
  [DllImport("user32.dll")] public static extern IntPtr SetThreadDpiAwarenessContext(IntPtr c);
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L, T, R, B; }
  [StructLayout(LayoutKind.Sequential)] public struct POINT { public int X, Y; }
}
"@
[void][W]::SetThreadDpiAwarenessContext([IntPtr](-4))
New-Item -ItemType Directory -Force $OutDir | Out-Null
function Find-Win {
  $script:best = [IntPtr]::Zero; $script:area = 0
  [W]::EnumWindows({ param($h, $l)
      $p = 0; [void][W]::GetWindowThreadProcessId($h, [ref]$p)
      if ($p -eq $ProcId -and [W]::IsWindowVisible($h)) {
        $sb = New-Object System.Text.StringBuilder 256; [void][W]::GetWindowText($h, $sb, 256)
        $r = New-Object W+RECT; [void][W]::GetWindowRect($h, [ref]$r)
        $a = ($r.R - $r.L) * ($r.B - $r.T)
        if ($sb.ToString().StartsWith("Fractadyne") -and $a -gt $script:area) { $script:best = $h; $script:area = $a }
      }
      return $true }, [IntPtr]::Zero) | Out-Null
  return $script:best
}
$sw = [Diagnostics.Stopwatch]::StartNew(); $h = [IntPtr]::Zero
while ($sw.Elapsed.TotalSeconds -lt $Seconds) {
  if ($h -eq [IntPtr]::Zero) { $h = Find-Win; if ($h -eq [IntPtr]::Zero) { Start-Sleep -Milliseconds 200; continue } }
  $wr = New-Object W+RECT; if (-not [W]::GetWindowRect($h, [ref]$wr)) { break }
  $ww = $wr.R - $wr.L; $wh = $wr.B - $wr.T
  $cr = New-Object W+RECT; [void][W]::GetClientRect($h, [ref]$cr)
  $pt = New-Object W+POINT; [void][W]::ClientToScreen($h, [ref]$pt)
  $bmp = New-Object System.Drawing.Bitmap $ww, $wh
  $g = [System.Drawing.Graphics]::FromImage($bmp); $hdc = $g.GetHdc()
  [void][W]::PrintWindow($h, $hdc, 2); $g.ReleaseHdc($hdc); $g.Dispose()
  $cw = $cr.R; $ch = $cr.B; $ox = $pt.X - $wr.L; $oy = $pt.Y - $wr.T
  $small = New-Object System.Drawing.Bitmap $cw, $ch
  $g2 = [System.Drawing.Graphics]::FromImage($small)
  $g2.DrawImage($bmp, (New-Object System.Drawing.Rectangle 0, 0, $small.Width, $small.Height), (New-Object System.Drawing.Rectangle $ox, $oy, $cw, $ch), [System.Drawing.GraphicsUnit]::Pixel)
  $g2.Dispose(); $bmp.Dispose()
  $small.Save((Join-Path $OutDir ("f{0:D6}.jpg" -f [int]$sw.ElapsedMilliseconds)), [System.Drawing.Imaging.ImageFormat]::Jpeg); $small.Dispose()
  Start-Sleep -Milliseconds $IntervalMs
}
