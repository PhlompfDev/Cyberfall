// Cyberfall telemetry, client half. One row per rendered frame.
global function CfTelemetryClient_Init

const string CF_COLUMNS = "tick,time,px,py,pz,vx,vy,vz,pitch,yaw,fwd,right,jump,crouch,sprint,ground,wallrun,hang,crouched\n"

const int CF_ROWS_PER_BLOCK = 64

struct
{
	bool recording = false
	// Set by the crouch key callbacks registered in CfTelemetryClient_Init.
	bool duckHeld = false
} file

void function CfTelemetryClient_Init()
{
	AddServerToClientStringCommandCallback( "cf_rec_client", CfClientCommand )
	// IN_DUCK never reports crouch, so watch the key (Left Ctrl).
	RegisterButtonPressedCallback( KEY_LCONTROL, CfDuckDown )
	RegisterButtonReleasedCallback( KEY_LCONTROL, CfDuckUp )
}

void function CfClientCommand( array<string> args )
{
	// Some builds include the command name as the first argument.
	if ( args.len() > 0 && args[0] == "cf_rec_client" )
		args.remove( 0 )
	if ( args.len() == 0 )
		return

	if ( args[0] == "start" && args.len() >= 2 && !file.recording )
		thread CfRecord( args[1] )
	else if ( args[0] == "stop" )
		file.recording = false
}

void function CfDuckDown( entity player )
{
	file.duckHeld = true
}

void function CfDuckUp( entity player )
{
	file.duckHeld = false
}

int function CfFlag( bool value )
{
	return value ? 1 : 0
}

void function CfRecord( string name )
{
	entity player = GetLocalClientPlayer()
	player.EndSignal( "OnDestroy" )

	file.recording = true
	string head = "# cyberfall-telemetry 1\n"
	head += "# map: " + GetMapName() + "\n"
	head += "# settings: " + player.GetPlayerSettings() + "\n"
	head += "# name: " + name + "\n"
	head += CF_COLUMNS

	array<string> blocks
	string block = ""
	int inBlock = 0
	int tick = 0

	printt( "[cyberfall] recording", name )

	while ( file.recording )
	{
		vector p = player.GetOrigin()
		vector v = player.GetVelocity()
		vector a = player.EyeAngles()

		block += format( "%d,%.4f,%.2f,%.2f,%.2f,%.2f,%.2f,%.2f,%.2f,%.2f,%.3f,%.3f,%d,%d,%d,%d,%d,%d,%d\n",
			tick, Time(),
			p.x, p.y, p.z,
			v.x, v.y, v.z,
			a.x, a.y,
			player.GetInputAxisForward(), player.GetInputAxisRight(),
			CfFlag( player.IsInputCommandHeld( IN_JUMP ) ),
			CfFlag( file.duckHeld ),
			CfFlag( player.IsInputCommandHeld( IN_SPEED ) ),
			CfFlag( player.IsOnGround() ),
			CfFlag( player.IsWallRunning() ),
			CfFlag( player.IsWallHanging() ),
			CfFlag( player.IsCrouched() ) )

		tick++
		inBlock++
		if ( inBlock == CF_ROWS_PER_BLOCK )
		{
			blocks.append( block )
			block = ""
			inBlock = 0
		}

		WaitFrame()
	}

	string text = head
	foreach ( string part in blocks )
		text += part
	text += block

	NSSaveFile( name + ".txt", text )
	printt( "[cyberfall] saved", name, "rows:", tick )
}
