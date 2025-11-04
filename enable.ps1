Remove-Item -Path "HKLM:\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate" -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item -Path "HKCU:\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate" -Recurse -Force -ErrorAction SilentlyContinue

if (Test-Path "HKLM:\SOFTWARE\Policies\Microsoft\WindowsStore") {
    Remove-ItemProperty -Path "HKLM:\SOFTWARE\Policies\Microsoft\WindowsStore" -Name "RemoveWindowsStore" -ErrorAction SilentlyContinue
    Remove-ItemProperty -Path "HKLM:\SOFTWARE\Policies\Microsoft\WindowsStore" -Name "DisableStoreApps" -ErrorAction SilentlyContinue
    Remove-ItemProperty -Path "HKLM:\SOFTWARE\Policies\Microsoft\WindowsStore" -Name "AutoDownload" -ErrorAction SilentlyContinue
}

gpupdate /target:computer /force
gpupdate /target:user /force

Set-Service -Name wuauserv -StartupType Manual
Set-Service -Name UsoSvc -StartupType Manual
Set-Service -Name WaaSMedicSvc -StartupType Manual
Set-Service -Name BITS -StartupType Manual
Set-Service -Name DoSvc -StartupType Automatic

Enable-ScheduledTask -TaskName "\Microsoft\Windows\WindowsUpdate\Scheduled Start" -ErrorAction SilentlyContinue
Enable-ScheduledTask -TaskName "\Microsoft\Windows\WindowsUpdate\AUScheduledInstall" -ErrorAction SilentlyContinue
Enable-ScheduledTask -TaskName "\Microsoft\Windows\WindowsUpdate\Automatic App Update" -ErrorAction SilentlyContinue
Enable-ScheduledTask -TaskName "\Microsoft\Windows\WindowsUpdate\Scheduled Start With Network" -ErrorAction SilentlyContinue
Enable-ScheduledTask -TaskName "\Microsoft\Windows\WindowsUpdate\UPR" -ErrorAction SilentlyContinue
Enable-ScheduledTask -TaskName "\Microsoft\Windows\UpdateOrchestrator\Schedule Scan" -ErrorAction SilentlyContinue
Enable-ScheduledTask -TaskName "\Microsoft\Windows\UpdateOrchestrator\Schedule Scan Static Task" -ErrorAction SilentlyContinue
Enable-ScheduledTask -TaskName "\Microsoft\Windows\UpdateOrchestrator\USO_UxBroker_Display" -ErrorAction SilentlyContinue
Enable-ScheduledTask -TaskName "\Microsoft\Windows\UpdateOrchestrator\UpdateModelTask" -ErrorAction SilentlyContinue
Enable-ScheduledTask -TaskName "\Microsoft\Windows\WaaSMedic\PerformRemediation" -ErrorAction SilentlyContinue
