require("exporter")

-- exporter.Start(listen_addr)
-- exporter.RegisterGauge(group_name,metric_name,description,label_key_1?,label_key_2?,label_key_x?)
-- exporter.UnregisterGroup(group_of_metrics_to_remove)

exporter.Start("0.0.0.0:9100")

exporter.UnregisterGroup("sourcesdk")
exporter.RegisterGauge("sourcesdk","sourcesdk_map","Current map","name")
exporter.SetGauge("sourcesdk_map",1,game.GetMap())

exporter.RegisterGauge("sourcesdk","sourcesdk_max_players","A total amount of players on server")
exporter.SetGauge("sourcesdk_max_players",game.MaxPlayers())

exporter.RegisterGauge("sourcesdk","sourcesdk_curtime","Current in-game time")
exporter.RegisterGauge("sourcesdk","sourcesdk_tickcount","Current tick count")
exporter.RegisterGauge("sourcesdk","sourcesdk_frametime","Server frametime")
exporter.RegisterGauge("sourcesdk","sourcesdk_player_count","A total amount of players on server")
exporter.RegisterGauge("sourcesdk","sourcesdk_visible_max_players","A total amount of players on server")
exporter.RegisterGauge("sourcesdk","sourcesdk_entities","A total amount of entities on server")
exporter.RegisterGauge("sourcesdk","sourcesdk_edicts","A total amount of edicts on server")

local sv_visiblemaxplayers = GetConVar("sv_visiblemaxplayers")
exporter.SetGauge("sourcesdk_visible_max_players",sv_visiblemaxplayers:GetInt())

cvars.AddChangeCallback("sv_visiblemaxplayers", function(_ , _, value)
    exporter.SetGauge("sourcesdk_visible_max_players",math.floor(value))
end)

timer.Create( "sourcesdk_exporter", 5, 0, function() 
    exporter.SetGauge("sourcesdk_curtime",CurTime()) 
    exporter.SetGauge("sourcesdk_tickcount",engine.TickCount()) 
    exporter.SetGauge("sourcesdk_frametime",FrameTime()) 
    exporter.SetGauge("sourcesdk_player_count",player.GetCount())
    exporter.SetGauge("sourcesdk_entities",ents.GetCount(true))
    exporter.SetGauge("sourcesdk_edicts",ents.GetEdictCount())
end) 

exporter.RegisterCounter("sourcesdk","sourcesdk_deaths","Player's deaths","steamid")
exporter.RegisterCounter("sourcesdk","sourcesdk_kills","Player's kills","steamid")

hook.Add("PlayerDeath","exporter_PlayerDeath",function(victim,_,attacker)
    if IsValid(victim) and victim:IsPlayer() then
        exporter.IncCounter("sourcesdk_deaths",victim:SteamID64())
    end
    if IsValid(attacker) and attacker:IsPlayer() then
        exporter.IncCounter("sourcesdk_kills",attacker:SteamID64())
    end
end)

hook.Add("PlayerDisconnected","exporter_PlayerDisconnected",function(player)
    exporter.RemoveCounter("sourcesdk_deaths",player:SteamID64())
    exporter.RemoveCounter("sourcesdk_kills",player:SteamID64())
end)

-- exporter.RegisterCounter(group_name,metric_name,description,label_key_1?,label_key_2?,label_key_x?)
exporter.RegisterCounter("glua","glua_errors","Garry's mod lua errors")

hook.Add( "OnLuaError", "glua_errors", function( error, realm, stack, name, addon_id )
    exporter.IncCounter("glua_errors")
end )