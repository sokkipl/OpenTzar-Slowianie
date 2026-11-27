using System;
using System.IO;
using System.IO.Compression;
using System.Net;
using System.Linq;
using System.Collections.Generic;
using System.Diagnostics;
using System.Drawing;
using System.Security.Cryptography;
using System.Threading.Tasks;
using System.Windows.Forms;
using System.Web.Script.Serialization;

public class Manifest {
 public string version {get;set;}
 public string full_url {get;set;}
 public string full_sha256 {get;set;}
 public string delta_from {get;set;}
 public string delta_url {get;set;}
 public string delta_sha256 {get;set;}
 public string notes {get;set;}
 public Dictionary<string,string> files {get;set;}
}
public class Launcher : Form {
 const string Channel = "https://github.com/sokkipl/OpenTzar-Slowianie/releases/download/launcher-channel/update.json";
 const string ReleasePrefix = "https://github.com/sokkipl/OpenTzar-Slowianie/releases/download/";
 static readonly string Root=Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),"OpenTzar","Player");
 static readonly string Game=Path.Combine(Root,"Game"), Stage=Path.Combine(Root,"Game.staging"), Backup=Path.Combine(Root,"Game.previous");
 const string StateName=".ot-launcher-manifest.json";
 static readonly JavaScriptSerializer Json=new JavaScriptSerializer {MaxJsonLength=16*1024*1024};
 Label current=new Label(), status=new Label(), online=new Label();
 TextBox notes=new TextBox();
 Button play=new Button(), update=new Button(), check=new Button(), repair=new Button();
 Manifest remote;
 bool busy;
 public Launcher() {
  Text="OpenTzar Launcher 1.0"; ClientSize=new Size(650,430); FormBorderStyle=FormBorderStyle.FixedDialog; MaximizeBox=false; StartPosition=FormStartPosition.CenterScreen;
  BackColor=Color.FromArgb(29,35,29); ForeColor=Color.FromArgb(236,228,203);
  var title=new Label{Text="OPEN TZAR",Font=new Font("Georgia",29,FontStyle.Bold),Location=new Point(24,18),AutoSize=true,ForeColor=Color.Wheat}; Controls.Add(title);
  AddLabel(current,80); AddLabel(online,111); AddLabel(status,145);
  notes.SetBounds(25,182,600,123); notes.Multiline=true; notes.ReadOnly=true; notes.ScrollBars=ScrollBars.Vertical; notes.BackColor=Color.FromArgb(42,49,42); notes.ForeColor=Color.Wheat; Controls.Add(notes);
  AddButton(play,"GRAJ",25,320,290); AddButton(update,"AKTUALIZUJ / ZAINSTALUJ",325,320,300); AddButton(check,"Sprawdź wersję",25,371,190); AddButton(repair,"Napraw grę",225,371,190);
  var folder=new Button{Text="Folder gry",Location=new Point(425,371),Size=new Size(200,40),FlatStyle=FlatStyle.Flat,ForeColor=Color.Wheat}; folder.Click+=(s,e)=>{Directory.CreateDirectory(Root);Process.Start("explorer.exe",Root);}; Controls.Add(folder);
  play.Click+=async(s,e)=>await RunJob(Play); check.Click+=async(s,e)=>await RunJob(Check); update.Click+=async(s,e)=>await RunJob(()=>Install(false)); repair.Click+=async(s,e)=>{if(MessageBox.Show("Pobrać i zweryfikować pełną paczkę gry?","Naprawa",MessageBoxButtons.YesNo)==DialogResult.Yes)await RunJob(()=>Install(true));};
  Shown+=async(s,e)=>await RunJob(Check); FormClosing+=(s,e)=>{if(busy){e.Cancel=true;MessageBox.Show("Trwa operacja. Spróbuj ponownie po jej zakończeniu.");}};
  Directory.CreateDirectory(Root); if(!Directory.Exists(Game)&&Directory.Exists(Backup))Directory.Move(Backup,Game);
  RefreshLabels();
 }
 void AddLabel(Label l,int y){l.SetBounds(25,y,600,29);l.ForeColor=Color.Wheat;l.Font=new Font("Segoe UI",10);Controls.Add(l);}
 void AddButton(Button b,string t,int x,int y,int w){b.Text=t;b.SetBounds(x,y,w,42);b.FlatStyle=FlatStyle.Flat;b.BackColor=Color.FromArgb(63,80,48);b.ForeColor=Color.Wheat;Controls.Add(b);}
 void Say(string s){if(InvokeRequired){Invoke(new Action<string>(Say),s);return;}status.Text=s;}
 void RefreshLabels(){var local=Local();current.Text="Zainstalowana: "+(local==null?"brak":local.version);online.Text="Dostępna: "+(remote==null?"nie sprawdzono":remote.version);notes.Text=remote==null?"Gra działa offline po pierwszej instalacji. Wymaga oryginalnego Tzara.":remote.notes;play.Enabled=!busy&&File.Exists(Path.Combine(Game,"OpenTzar.exe"));update.Enabled=!busy&&remote!=null&&(local==null||local.version!=remote.version);repair.Enabled=!busy&&remote!=null;check.Enabled=!busy;}
 async Task RunJob(Action action){if(busy)return;busy=true;RefreshLabels();try{await Task.Run(action);}catch(Exception ex){Say("Błąd: "+ex.Message);}finally{busy=false;RefreshLabels();}}
 static WebClient Client(){var c=new WebClient();c.Encoding=System.Text.Encoding.UTF8;c.Headers["User-Agent"]="OpenTzar-Launcher/1.0";return c;}
 static string Safe(string root,string relative){if(String.IsNullOrEmpty(relative)||relative.Contains(":")||relative.Contains("\\")||relative.StartsWith("/")||relative.Split('/').Any(s=>s==""||s=="."||s==".."))throw new IOException("Nieprawidłowa ścieżka w aktualizacji.");string p=Path.GetFullPath(Path.Combine(root,relative.Replace('/',Path.DirectorySeparatorChar)));if(!p.StartsWith(Path.GetFullPath(root)+Path.DirectorySeparatorChar,StringComparison.OrdinalIgnoreCase))throw new IOException("Ścieżka poza katalogiem gry.");return p;}
 static string Hash(string p){using(var s=File.OpenRead(p))using(var h=SHA256.Create())return BitConverter.ToString(h.ComputeHash(s)).Replace("-","").ToLowerInvariant();}
 static bool Match(string p,string h){return File.Exists(p)&&String.Equals(Hash(p),h,StringComparison.OrdinalIgnoreCase);}
 static void ValidateUrl(string u){if(String.IsNullOrEmpty(u)||!u.StartsWith(ReleasePrefix,StringComparison.Ordinal))throw new IOException("Nieprawidłowy adres aktualizacji.");}
 static Manifest Parse(string s){var m=Json.Deserialize<Manifest>(s);if(m==null||String.IsNullOrWhiteSpace(m.version)||m.files==null||!m.files.ContainsKey("OpenTzar.exe")||m.files.ContainsKey(StateName))throw new IOException("Nieprawidłowy manifest.");ValidateUrl(m.full_url);foreach(var kv in m.files){Safe(Game,kv.Key);if(kv.Value==null||kv.Value.Length!=64||!kv.Value.All(Uri.IsHexDigit))throw new IOException("Nieprawidłowa suma kontrolna.");}if(!String.IsNullOrEmpty(m.delta_url))ValidateUrl(m.delta_url);return m;}
 static Manifest Local(){try{return Parse(File.ReadAllText(Path.Combine(Game,StateName)));}catch{return null;}}
 void Check(){Say("Sprawdzanie GitHuba...");using(var c=Client()){var m=Parse(c.DownloadString(Channel));remote=m;}Say("Sprawdzono wersję. Możesz grać także bez internetu.");}
 static void CopyDir(string from,string to){Directory.CreateDirectory(to);foreach(var f in Directory.GetFiles(from)){File.Copy(f,Path.Combine(to,Path.GetFileName(f)),true);}foreach(var d in Directory.GetDirectories(from)){if((File.GetAttributes(d)&FileAttributes.ReparsePoint)!=0)throw new IOException("Niedozwolony link w folderze gry.");CopyDir(d,Path.Combine(to,Path.GetFileName(d)));}}
 static void Extract(string zip,string root,Manifest m){using(var a=ZipFile.OpenRead(zip)){foreach(var e in a.Entries){if(String.IsNullOrEmpty(e.Name))continue;string n=e.FullName.Replace('\\','/');if(!m.files.ContainsKey(n))throw new IOException("Nieoczekiwany plik: "+n);string p=Safe(root,n);Directory.CreateDirectory(Path.GetDirectoryName(p));e.ExtractToFile(p,true);}}}
 static void Verify(string root,Manifest m){foreach(var kv in m.files)if(!Match(Safe(root,kv.Key),kv.Value))throw new IOException("Uszkodzony lub brakujący plik: "+kv.Key);}
 void Install(bool full){if(Process.GetProcessesByName("OpenTzar").Length>0)throw new IOException("Najpierw zamknij grę.");if(remote==null)Check();var m=remote;var old=Local();bool delta=!full&&old!=null&&old.version==m.delta_from&&!String.IsNullOrEmpty(m.delta_url);
  if(delta){try{Verify(Game,old);}catch{delta=false;}}
  string zip=Path.Combine(Root,"download.tmp.zip");
  try{
   Say(delta?"Pobieranie zmienionych plików...":"Pobieranie pełnej gry...");
   using(var c=Client())c.DownloadFile(delta?m.delta_url:m.full_url,zip);
   if(!Match(zip,delta?m.delta_sha256:m.full_sha256))throw new IOException("Suma kontrolna ZIP jest niezgodna.");
   Say("Instalowanie i sprawdzanie plików...");
   if(Directory.Exists(Stage))Directory.Delete(Stage,true); Directory.CreateDirectory(Stage);
   if(Directory.Exists(Game))CopyDir(Game,Stage);
   if(old!=null)foreach(var k in old.files.Keys)if(!m.files.ContainsKey(k)){var p=Safe(Stage,k);if(File.Exists(p))File.Delete(p);}
   Extract(zip,Stage,m);Verify(Stage,m);File.WriteAllText(Path.Combine(Stage,StateName),Json.Serialize(m));
   if(Directory.Exists(Backup))Directory.Delete(Backup,true);
   if(Directory.Exists(Game))Directory.Move(Game,Backup);
   try{Directory.Move(Stage,Game);}catch{if(!Directory.Exists(Game)&&Directory.Exists(Backup))Directory.Move(Backup,Game);throw;}
   Say("Zainstalowano "+m.version+". Poprzednia wersja pozostaje w kopii zapasowej.");
  }finally{if(File.Exists(zip))File.Delete(zip);if(Directory.Exists(Stage))Directory.Delete(Stage,true);}
 }
 void Play(){var exe=Path.Combine(Game,"OpenTzar.exe");if(!File.Exists(exe))throw new IOException("Najpierw zainstaluj grę.");if(Process.GetProcessesByName("OpenTzar").Length>0)throw new IOException("Gra jest już uruchomiona.");Say("Sprawdzanie zasobów...");using(var p=Process.Start(new ProcessStartInfo(exe,"--ot-check-assets"){WorkingDirectory=Game,UseShellExecute=false})){p.WaitForExit();if(p.ExitCode!=0)throw new IOException("Kontrola zasobów nie powiodła się. Użyj Napraw grę.");}
  Process.Start(new ProcessStartInfo(exe,"--root . --skip-intro"){WorkingDirectory=Game,UseShellExecute=false});Say("Uruchomiono grę.");}
 [STAThread] public static void Main(){ServicePointManager.SecurityProtocol=SecurityProtocolType.Tls12;bool created;using(var mutex=new System.Threading.Mutex(true,"Local\\OpenTzarLauncherPlayer",out created)){if(!created){MessageBox.Show("Launcher jest już uruchomiony.");return;}Application.EnableVisualStyles();Application.SetCompatibleTextRenderingDefault(false);Application.Run(new Launcher());}}
}
