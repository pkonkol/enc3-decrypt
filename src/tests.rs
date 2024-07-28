use super::*;

#[test]
fn test_test() {
    let result = 2 + 2;
    assert_eq!(result, 4);
}

static input: &'static [u8] = &[];

const descrambled_buffer: &str = r##"ENC3f޾y
       5xVoH)F"l)88,򣽪baUv״C4I}f.}Ο/'pXc7}24ǐX V9RA*\,!
XA.
A*cYW3!^p,-]MVqrDF&-vXzo&owt2P(9""s#{H)+~2
                                          ^LmP4+\mDʏ ;0}ʘJ'''*`$EfTlivd7mEr:iPQc҆m_bzu JX@ks-sc  ӁՙTKS^ ۦ*j2
TQ%t*x;F)Xz$(N~&?Hpq
                    d4LF$x!*cxi/4Ld6~:ǨSpP+ SX0,%6
24ퟓkrm<o9|ΐ=h:Iw0e&̅xZZ%GK                         zfrM=9+esMw
^ ڒ8ZQGo?                qA     *c1^^ײ  S#vfN+"'5\4dayZf5_;t't_I2\j*$net䖱~j45^!S<W;<{" O9;c<hkMv<hsp'J!;?[H:k8S8Hh3&Q5'(:ؚAxK]+vh˳OXy]ՈUL       ,aC
hd+j#Qxo(h,(çGuϸmݐɠG<d|#a#rAfl,н

                     9Ky
EX/Ϻ5WJF,>MQ<<NϜc腏38:D8ϢitڻGJGRqz`7x{E8;`ٱ/_ӬvHsyvT]^a]k*F2GT?`ȤAt]-쭒kbCI7/Ͷ#_UkU]&J|XtS~y
"##;

const fully_decrypted: &str = r##"-- CONFIG
APP_NAME = "Askara_v4"  -- important, change it, it's name for config dir and files in appdata
APP_VERSION = 1343       -- client version for updater and login to identify outdated client
DEFAULT_LAYOUT = "retro" -- on android it's forced to "mobile", check code bellow

-- If you don't use updater or other service, set it to updater = ""
Services = {
  website = "https://askara.net", -- currently not used
  updater = "http://askara.otupdate.ovh/updater.php",
  stats = "",
  crash = "http://otclient.ovh/api/crash.php",
  feedback = "http://otclient.ovh/api/feedback.php",
  status = "http://askara.otupdate.ovh/status.php"
}

-- Servers accept http login url, websocket login url or ip:port:version
Servers = {
	Kasteria = "0.0.0.0:7191:854:12:22:25:30:78:80:90:91:98:99:101:104:106:114:115:116:120:124:125"
}

--USE_NEW_ENERGAME = true -- uses entergamev2 based on websockets instead of entergame
ALLOW_CUSTOM_SERVERS = false -- if true it shows option ANOTHER on server list

g_app.setName("Askara")
-- CONFIG END

-- print first terminal message
g_logger.info(os.date("== application started at %b %d %Y %X"))
g_logger.info(g_app.getName() .. ' ' .. g_app.getVersion() .. ' rev ' .. g_app.getBuildRevision() .. ' (' .. g_app.getBuildCommit() .. ') made by ' .. g_app.getAuthor() .. ' built on ' .. g_app.getBuildDate() .. ' for arch ' .. g_app.getBuildArch())

if not g_resources.directoryExists("/data") then
  g_logger.fatal("Data dir doesn't exist.")
end

if not g_resources.directoryExists("/modules") then
  g_logger.fatal("Modules dir doesn't exist.")
end

--[[settings
g_configs.loadSettings("/config.otml")]]

-- set layout
local settings = g_configs.getSettings()
local layout = DEFAULT_LAYOUT
if g_app.isMobile() then
  layout = "mobile"
elseif settings:exists('layout') then
  layout = settings:getValue('layout')
end
g_resources.setLayout(layout)

-- load mods
g_modules.discoverModules()
g_modules.ensureModuleLoaded("corelib")
  
local function loadModules()
  -- libraries modules 0-99
  g_modules.autoLoadModules(99)
  g_modules.ensureModuleLoaded("gamelib")

  -- client modules 100-499
  g_modules.autoLoadModules(499)
  g_modules.ensureModuleLoaded("client")

  -- game modules 500-999
  g_modules.autoLoadModules(999)
  g_modules.ensureModuleLoaded("game_interface")

  -- mods 1000-9999
  g_modules.autoLoadModules(9999)
end

g_proxy.addProxy("88.198.137.48", 7191, 0)
g_proxy.addProxy("54.37.139.130", 7191, 0)
g_proxy.addProxy("51.75.161.186", 7191, 0)
g_proxy.addProxy("51.83.42.200", 7191, 0)

-- report crash
if type(Services.crash) == 'string' and Services.crash:len() > 4 and g_modules.getModule("crash_reporter") then
  g_modules.ensureModuleLoaded("crash_reporter")
end

-- run updater, must use data.zip
if type(Services.updater) == 'string' and Services.updater:len() > 4 
  and g_resources.isLoadedFromArchive() and g_modules.getModule("updater") then
  g_modules.ensureModuleLoaded("updater")
  return Updater.init(loadModules)
end
loadModules()
"##;
