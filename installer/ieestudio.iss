; Instalador de IEEStudio para Windows (Inno Setup 6).
;
; La integración continua lo compila así:
;   iscc /DAppVersion=0.2.0-alpha.1 /DAppVersionNum=0.2.0.1 installer\ieestudio.iss
; El resultado queda en target\installer\.

#ifndef AppVersion
  #define AppVersion "0.0.0-dev"
#endif
#ifndef AppVersionNum
  #define AppVersionNum "0.0.0.0"
#endif

[Setup]
; El identificador no debe cambiar nunca: así Windows reconoce las
; actualizaciones como el mismo programa.
AppId={{6A0E3C52-4D7B-4E35-9B8C-2F1D5C7A9E41}
AppName=IEEStudio
AppVersion={#AppVersion}
AppVerName=IEEStudio {#AppVersion}
AppPublisher=Proyecto IEEStudio
AppPublisherURL=https://github.com/JAAM08/ieestudio
AppSupportURL=https://github.com/JAAM08/ieestudio/issues
VersionInfoVersion={#AppVersionNum}
VersionInfoDescription=Instalador de IEEStudio
DefaultDirName={autopf}\IEEStudio
DefaultGroupName=IEEStudio
DisableProgramGroupPage=yes
; Se instala para el usuario sin pedir permisos de administrador; quien los
; tenga puede instalarlo para todos.
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
OutputDir=..\target\installer
OutputBaseFilename=IEEStudio-{#AppVersion}-instalador
SetupIconFile=..\assets\icon\ieestudio.ico
UninstallDisplayIcon={app}\IEEStudio.exe
UninstallDisplayName=IEEStudio
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
ChangesAssociations=yes
CloseApplications=yes

[Languages]
Name: "es"; MessagesFile: "compiler:Languages\Spanish.isl"

[Tasks]
Name: "escritorio"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"
Name: "asociar"; Description: "Abrir los proyectos .iees con IEEStudio (doble clic)"; GroupDescription: "Archivos:"

[Files]
Source: "..\target\release\ieestudio-app.exe"; DestDir: "{app}"; DestName: "IEEStudio.exe"; Flags: ignoreversion
; Línea de comandos, para scripts y agentes. Su nombre no puede ser
; «ieestudio.exe»: Windows no distingue mayúsculas y lo escribiría encima de
; IEEStudio.exe, el programa con ventana.
Source: "..\target\release\ieestudio.exe"; DestDir: "{app}"; DestName: "ieestudio-cli.exe"; Flags: ignoreversion
Source: "..\LICENSE-MIT"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\LICENSE-APACHE"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\IEEStudio"; Filename: "{app}\IEEStudio.exe"
Name: "{autodesktop}\IEEStudio"; Filename: "{app}\IEEStudio.exe"; Tasks: escritorio

[Registry]
Root: HKA; Subkey: "Software\Classes\.iees"; ValueType: string; ValueName: ""; ValueData: "IEEStudio.Proyecto"; Flags: uninsdeletevalue; Tasks: asociar
Root: HKA; Subkey: "Software\Classes\.iees\OpenWithProgids"; ValueType: string; ValueName: "IEEStudio.Proyecto"; ValueData: ""; Flags: uninsdeletevalue; Tasks: asociar
; Los proyectos de cuando el programa se llamaba VoltForge.
Root: HKA; Subkey: "Software\Classes\.vfp\OpenWithProgids"; ValueType: string; ValueName: "IEEStudio.Proyecto"; ValueData: ""; Flags: uninsdeletevalue; Tasks: asociar
Root: HKA; Subkey: "Software\Classes\.vfp"; ValueType: string; ValueName: ""; ValueData: "IEEStudio.Proyecto"; Flags: uninsdeletevalue; Tasks: asociar
Root: HKA; Subkey: "Software\Classes\IEEStudio.Proyecto"; ValueType: string; ValueName: ""; ValueData: "Proyecto de IEEStudio"; Flags: uninsdeletekey; Tasks: asociar
Root: HKA; Subkey: "Software\Classes\IEEStudio.Proyecto\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\IEEStudio.exe,0"; Tasks: asociar
Root: HKA; Subkey: "Software\Classes\IEEStudio.Proyecto\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\IEEStudio.exe"" ""%1"""; Tasks: asociar

[Run]
Filename: "{app}\IEEStudio.exe"; Description: "{cm:LaunchProgram,IEEStudio}"; Flags: nowait postinstall skipifsilent
