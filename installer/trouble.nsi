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

!define MUI_ABORTWARNING
!define MUI_ICON "..\icons\icon.ico"
!define MUI_UNICON "..\icons\icon.ico"

!insertmacro MUI_PAGE_LICENSE "..\LICENSE"
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "English"

Section "Trouble" SecMain
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
