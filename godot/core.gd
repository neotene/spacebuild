extends Node

enum State {INIT, WELCOME, LOADING, PLAYING}
enum NetworkState {IDLE, CONNECTING, AUTHENTICATING}
enum ServerProcessState {NOT_RUNNING, RUNNING, READY}
enum PlayMode {SOLO_CREATION, SOLO_JOIN, ONLINE}

var socket = WebSocketPeer.new()
var login_hash = {
	"Login": {
		"nickname": ""
	}
}
var login = ""
var server = null;
var state = State.INIT
var network_state = NetworkState.IDLE
var server_process_state = ServerProcessState.NOT_RUNNING
var server_logs_thread: Thread
@onready var server_logs = $"../2D/Server" as RichTextLabel

@onready var ui = $"../2D/UI"

func _ready() -> void:
	pass

func _server_logs():
	var pipe_err = server["stderr"] as FileAccess

	while server_process_state == ServerProcessState.RUNNING:		
		var stderr_line = pipe_err.get_line()
		print(stderr_line)
		if !OS.has_feature("release"):
			server_logs.call_deferred("append_text", stderr_line)
			server_logs.call_deferred("newline")

func refresh(to_state) -> void:
	#ui.error_placeholder.set_text("Connecting...")
	$"../2D/UI".set_visible(to_state == State.WELCOME)
	$"../2D/Title".set_visible(to_state == State.WELCOME)
	$"../2D/Loading".set_text("Connecting...")
	$"../2D/Loading".set_visible(to_state == State.LOADING)
	state = to_state

func _process(_delta: float) -> void:
	if state == State.INIT:
		state = State.WELCOME
		return

	if server_process_state == ServerProcessState.RUNNING:
		if !OS.is_process_running(server["pid"]):
			server_process_state = ServerProcessState.NOT_RUNNING
			refresh(State.WELCOME)
			server_logs_thread.wait_to_finish()

	if network_state == NetworkState.CONNECTING:
		socket.poll()
		var socket_state = socket.get_ready_state()
		if socket_state == WebSocketPeer.STATE_OPEN:
			network_state = NetworkState.AUTHENTICATING
			login_hash["Login"] = ui.login_field.get_text()
			socket.send_text(JSON.stringify(login_hash))
		elif socket_state == WebSocketPeer.STATE_CLOSING:
			socket.poll()
			print("Closing")
			pass
		elif socket_state == WebSocketPeer.STATE_CLOSED:
			var code = socket.get_close_code()
			var reason = socket.get_close_reason()
			var error_str = "Could not connect"
			if !reason.is_empty():
				error_str += ": %s" % reason
			ui.error_placeholder.set_text(error_str)
			print("WebSocket closed with code: %d, reason %s. Clean: %s" % [code, reason, code != -1])
			network_state = NetworkState.IDLE

	if network_state == NetworkState.AUTHENTICATING:
		socket.poll()
		while socket.get_available_packet_count():
			var variant = JSON.parse_string(socket.get_packet().get_string_from_utf8())
			print(variant)
	
func quit() -> void:
	if server_process_state == ServerProcessState.RUNNING:
		OS.kill(server["pid"])
	get_tree().quit()

func play_solo(play_mode) -> String:
	var _output = []
	OS.set_environment("RUST_LOG", "INFO")
	var world_text = ""
	if play_mode == PlayMode.SOLO_CREATION:
		world_text = ui.world_field.get_text()
	elif play_mode == PlayMode.SOLO_JOIN:
		world_text = ui.worlds_tree.get_selected().get_text(0)
		
	assert(!world_text.is_empty())
	var args = ["0", "--no-input", "--instance", ProjectSettings.globalize_path("user://%s.spdb" % world_text)]
	if !OS.has_feature("release"):
		server = OS.execute_with_pipe("../target/debug/spacebuild-server", args)
	else:
		server = OS.execute_with_pipe("./spacebuild-server", args)
	if server.is_empty():
		printerr("Failed to run server")
		ui.error_placeholder.set_text("Local server failure")
		ui.play_button.set_disabled(false)
		return ""
	server_process_state = ServerProcessState.RUNNING
	server_logs_thread = Thread.new()
	server_logs_thread.start(_server_logs)
	return "ws://localhost:2567"

func play_online() -> String:
	login = ui.login_field.get_text()
	if login.is_empty():
		ui.error_placeholder.set_text("Enter your login please")
		print("No login")
		ui.play_button.set_disabled(false)
		return ""

	var host_str = ui.host_field.get_text()
	if host_str.is_empty():
		host_str = "localhost"
		
	var port_str = ui.port_field.get_text()
	if port_str.is_empty():
		port_str = "2567"
		
	var protocol_str = "wss" if ui.encrypted_switch.is_pressed() else "ws"
	
	return "%s://%s:%s" % [protocol_str, host_str, port_str]

func play(play_mode) -> void:
	
	var uri = ""

	if play_mode == PlayMode.SOLO_CREATION || play_mode == PlayMode.SOLO_JOIN:
		uri = play_solo(play_mode)
	elif play_mode == PlayMode.ONLINE:
		uri = play_online()

	if uri.is_empty():
		return 

	print("Connecting to %s..." % uri)
	socket = WebSocketPeer.new()
	
	var options = TLSOptions.client()
	if ui.welcome_state == ui.WelcomeState.ONLINE && ui.encrypted_switch.is_pressed():
		var cert = X509Certificate.new()
		if cert.load("ca_cert.pem") == OK:
			options = TLSOptions.client(cert)
	
	if socket.connect_to_url(uri, options) != OK:
		printerr("Could not connect")
		ui.error_placeholder.set_text("Could not connect")
		ui.play_button.set_disabled(false)
		return 
	
	refresh(State.LOADING)
	network_state = NetworkState.CONNECTING
