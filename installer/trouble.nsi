; NSIS installer script for Trouble.
; Build with makensis (https://nsis.sourceforge.io/) from a Windows shell, after a release
; build has produced target\release\trouble.exe:
;
;   makensis /DVERSION=0.5.0 installer\trouble.nsi
;
; Produces TroubleSetup-<VERSION>.exe in the repo root.

!ifndef VERSION
  !define VERSION "0.0.0"
!endif

Name "Trouble"
OutFile "TroubleSetup-${VERSION}.exe"
InstallDir "$PROGRAMFILES64\Trouble"
InstallDirRegKey HKCU "Software\Trouble" "InstallDir"
RequestExecutionLevel admin

!include "MUI2.nsh"
!include "Sections.nsh"

!define MUI_ABORTWARNING
!define MUI_ICON "..\icons\icon.ico"
!define MUI_UNICON "..\icons\icon.ico"
!define MUI_COMPONENTSPAGE_NODESC

!insertmacro MUI_PAGE_LICENSE "..\LICENSE"
!insertmacro MUI_PAGE_COMPONENTS
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "English"

; trouble.exe is built against the MSVC toolchain and dynamically links the
; Visual C++ runtime. Most Windows installs already have it, but a fresh
; machine without Visual Studio or a prior VC++-dependent app installed can be
; missing it, which makes trouble.exe fail to launch with error 126. Checked
; on .onInit, before the Components page is shown: if present, check and gray
; out the component so the user can see it's already covered (and skip
; re-running it); if absent, leave it checked but selectable.
Var VCRedistInstalled

Section "Trouble" SecMain
  SectionIn RO
  SetOutPath "$INSTDIR"
  File "..\target\release\trouble.exe"

  WriteRegStr HKCU "Software\Trouble" "InstallDir" "$INSTDIR"
  WriteUninstaller "$INSTDIR\Uninstall.exe"

  CreateDirectory "$SMPROGRAMS\Trouble"
  CreateShortCut "$SMPROGRAMS\Trouble\Trouble.lnk" "$INSTDIR\trouble.exe"
  CreateShortCut "$SMPROGRAMS\Trouble\Uninstall.lnk" "$INSTDIR\Uninstall.exe"
  CreateShortCut "$DESKTOP\Trouble.lnk" "$INSTDIR\trouble.exe"

  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Trouble" \
    "DisplayName" "Trouble"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Trouble" \
    "UninstallString" "$INSTDIR\Uninstall.exe"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Trouble" \
    "DisplayIcon" "$INSTDIR\trouble.exe"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Trouble" \
    "DisplayVersion" "${VERSION}"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Trouble" \
    "Publisher" "Trouble"
  WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Trouble" \
    "NoModify" 1
  WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Trouble" \
    "NoRepair" 1
SectionEnd

Section "Visual C++ Redistributable" SecVCRedist
  StrCmp $VCRedistInstalled "1" VCRedistSectionDone
  DetailPrint "Downloading Visual C++ Redistributable (required to run Trouble)..."
  SetOutPath "$TEMP"
  NSISdl::download "https://aka.ms/vs/17/release/vc_redist.x64.exe" "$TEMP\vc_redist.x64.exe"
  Pop $0
  StrCmp $0 "success" 0 VCRedistDownloadFailed
    DetailPrint "Installing Visual C++ Redistributable, this may take a minute..."
    ExecWait '"$TEMP\vc_redist.x64.exe" /install /quiet /norestart'
    Delete "$TEMP\vc_redist.x64.exe"
    DetailPrint "Visual C++ Redistributable installed."
    Goto VCRedistSectionDone
  VCRedistDownloadFailed:
    DetailPrint "Failed to download Visual C++ Redistributable."
    MessageBox MB_OK|MB_ICONEXCLAMATION "Could not download the Visual C++ Redistributable. If Trouble fails to launch, install it manually from https://aka.ms/vs/17/release/vc_redist.x64.exe"
  VCRedistSectionDone:
SectionEnd

Function .onInit
  ClearErrors
  ReadRegDWORD $0 HKLM "SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\X64" "Installed"
  IfErrors VCRedistMissing
    StrCpy $VCRedistInstalled "1"
    SectionSetFlags ${SecVCRedist} ${SF_SELECTED}|${SF_RO}
    SectionSetText ${SecVCRedist} "Visual C++ Redistributable (already installed)"
    Goto VCRedistInitDone
  VCRedistMissing:
    StrCpy $VCRedistInstalled "0"
    SectionSetFlags ${SecVCRedist} ${SF_SELECTED}
  VCRedistInitDone:
FunctionEnd

Section "Uninstall"
  Delete "$INSTDIR\trouble.exe"
  Delete "$INSTDIR\Uninstall.exe"
  RMDir "$INSTDIR"

  Delete "$SMPROGRAMS\Trouble\Trouble.lnk"
  Delete "$SMPROGRAMS\Trouble\Uninstall.lnk"
  RMDir "$SMPROGRAMS\Trouble"
  Delete "$DESKTOP\Trouble.lnk"

  DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Trouble"
  DeleteRegKey HKCU "Software\Trouble"
SectionEnd
