; Script généré pour WikiTools (Native Edition)
; Nécessite Inno Setup pour être compilé

#define MyAppName "WikiTools"
#define MyAppVersion "1.0"
#define MyAppPublisher "Romain Perso"
#define MyAppExeName "WikiTools.exe"

[Setup]
; NOTE: The value of AppId uniquely identifies this application.
; Do not use the same AppId value in installers for other applications.
; (To generate a new GUID, click Tools | Generate GUID inside the IDE.)
AppId={{C6912759-EF89-4711-AF0C-F9F9D81A824D}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
DefaultDirName={autopf}\{#MyAppName}
DisableProgramGroupPage=yes
; On demande les droits admin pour écrire dans Program Files
PrivilegesRequired=admin
OutputDir=.
OutputBaseFilename=WikiTools_Setup_v1.0
Compression=lzma
SolidCompression=yes
WizardStyle=modern

[Languages]
Name: "french"; MessagesFile: "compiler:Languages\French.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
; 1. L'exécutable principal
Source: "WikiTools.exe"; DestDir: "{app}"; Flags: ignoreversion

; 2. Le dossier du serveur Wiki.js (Node.js)
Source: "wiki\*"; DestDir: "{app}\wiki"; Flags: ignoreversion recursesubdirs createallsubdirs

; 3. Le moteur PostgreSQL (Binaires Portables)
; C'est ici que la magie opère : on livre le moteur avec l'appli !
Source: "postgresql\*"; DestDir: "{app}\postgresql"; Flags: ignoreversion recursesubdirs createallsubdirs

; 4. Les assets (logos etc) si besoin (déjà dans l'exe ou wiki ?)
Source: "assets\*"; DestDir: "{app}\assets"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{autoprograms}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent
