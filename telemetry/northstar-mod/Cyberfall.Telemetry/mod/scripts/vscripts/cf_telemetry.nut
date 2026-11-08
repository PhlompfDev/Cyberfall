// Cyberfall telemetry, server half. Relays commands to the client script.
// Console: cf_rec start <name> | cf_rec stop | cf_rec settings
global function CfTelemetry_Init

const array<string> CF_SETTINGS = [
	"jumpheight", "gravityscale", "stepheight",
	"wallrun_timeLimit", "wallrunJumpOutwardSpeed", "wallrunJumpUpSpeed",
	"wallrunJumpInputDirSpeed", "wallrunMaxSpeedVertical", "wallrunMaxSpeedHorizontal",
	"wallrunAccelerateVertical", "wallrunAccelerateHorizontal",
	"slidedecel", "slidevelocitydecay",
	"superjumpMinHeight", "superjumpMaxHeight", "superjumpHorzSpeed",
	"sprintStartDelay", "sprintStartDuration"
]

void function CfTelemetry_Init()
{
	AddClientCommandCallback( "cf_rec", ClientCommand_CfRec )
}

bool function ClientCommand_CfRec( entity player, array<string> args )
{
	if ( args.len() == 0 )
		return true

	if ( args[0] == "start" && args.len() >= 2 )
		ServerToClientStringCommand( player, "cf_rec_client start " + args[1] )
	else if ( args[0] == "stop" )
		ServerToClientStringCommand( player, "cf_rec_client stop" )
	else if ( args[0] == "settings" )
		CfDumpSettings( player )

	return true
}

void function CfDumpSettings( entity player )
{
	string text = "key,value\n"
	text += "settings," + player.GetPlayerSettings() + "\n"
	foreach ( string key in CF_SETTINGS )
		text += key + "," + player.GetPlayerSettingsField( key ) + "\n"

	NSSaveFile( "settings.txt", text )
	printt( "[cyberfall] saved settings.txt" )
}
