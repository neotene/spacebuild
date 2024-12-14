extends Node

enum State {INIT, WELCOME, WAITING_PORT, LOADING, PLAYING}
enum NetworkState {IDLE, CONNECTING, AUTHENTICATING, WAITING_GAMEINFO}
enum ServerProcessState {NOT_RUNNING, RUNNING, READY}
enum PlaySoloMode {CREATION, JOIN}

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
var mutex: Mutex = Mutex.new()
var server_port = 0
var server_uri: String = ""

@onready var server_logs = get_tree().get_first_node_in_group("server_logs")


@onready var ui = get_tree().get_first_node_in_group("ui")
var regex = RegEx.new()

func _notification(what):
	if what == NOTIFICATION_WM_CLOSE_REQUEST:
		quit()

func _ready() -> void:
	regex.compile("^.*Server loop starts, listenning on (\\d+)$")

func _server_logs():
	var pipe_err = server["stderr"] as FileAccess

	while server_process_state == ServerProcessState.RUNNING:		
		var stderr_line = pipe_err.get_line()
		if pipe_err.eof_reached() || stderr_line.is_empty():
			return
		var search_result = regex.search(stderr_line)
		if search_result:
			var port_str = search_result.get_string(1)
			assert(!port_str.is_empty())
			mutex.lock()
			server_port = int(port_str)
			mutex.unlock()
		print("Server says: [%s]" % stderr_line)
		if !OS.has_feature("release"):
			server_logs.call_deferred("append_text", stderr_line)
			server_logs.call_deferred("newline")

func refresh(to_state, to_network_state) -> void:
	#ui.error_placeholder.set_text("Connecting...")
	get_tree().get_first_node_in_group("modale").set_visible(to_state == State.WELCOME)
	get_tree().get_first_node_in_group("title").set_visible(to_state == State.WELCOME)
	get_tree().get_first_node_in_group("loading").set_visible(to_state == State.WAITING_PORT
								 || to_state == State.LOADING)
	if to_state == State.WAITING_PORT:
		get_tree().get_first_node_in_group("loading").set_text("Wait please...")
	elif to_state == State.LOADING:
		get_tree().get_first_node_in_group("loading").set_text("Connecting...")
	state = to_state
	network_state = to_network_state

func _process(_delta: float) -> void:
	if state == State.INIT:
		state = State.WELCOME
		return

	if state == State.WAITING_PORT:
		mutex.lock()
		var port = server_port
		mutex.unlock()
		if port:
			server_uri += ":%d" % server_port
			print("Connecting to %s..." % server_uri)
			socket = WebSocketPeer.new()
			
			var options = TLSOptions.client()
			if ui.welcome_state == ui.WelcomeState.ONLINE && ui.encrypted_switch.is_pressed():
				var cert = X509Certificate.new()
				if cert.load("ca_cert.pem") == OK:
					options = TLSOptions.client(cert)
			
			if socket.connect_to_url(server_uri, options) != OK:
				printerr("Could not connect")
				ui.error_placeholder.set_text("Could not connect")
				ui.play_button.set_disabled(false)
				refresh(State.WELCOME, network_state)
				return 
			
			refresh(State.LOADING, NetworkState.CONNECTING)


	if server_process_state == ServerProcessState.RUNNING:
		if !OS.is_process_running(server["pid"]):
			server_process_state = ServerProcessState.NOT_RUNNING
			refresh(State.WELCOME, network_state)
			server_logs_thread.wait_to_finish()

	if network_state != NetworkState.IDLE:
		socket.poll()
		var socket_state = socket.get_ready_state()
		if socket_state == WebSocketPeer.STATE_CLOSING:
			socket.poll()
			print("Closing")
			pass
		elif socket_state == WebSocketPeer.STATE_CLOSED:
			var code = socket.get_close_code()
			var reason = socket.get_close_reason()
			#var error_str = "Could not connect"
			#if !reason.is_empty():
				#error_str += ": %s" % reason
			#ui.error_placeholder.set_text(error_str)
			print("WebSocket closed with code: %d, reason %s. Clean: %s" % [code, reason, code != -1])
			network_state = NetworkState.IDLE

		if network_state == NetworkState.CONNECTING:
			if socket_state == WebSocketPeer.STATE_OPEN:
				network_state = NetworkState.AUTHENTICATING
				login_hash["Login"]["nickname"] = ui.login_field.get_text()
				socket.send_text(JSON.stringify(login_hash))
		elif network_state == NetworkState.AUTHENTICATING:
			while socket.get_available_packet_count():
				var variant = JSON.parse_string(socket.get_packet().get_string_from_utf8())
				print("Received: [%s]" % variant)
				if variant["success"] == false:
					ui.error_placeholder.set_text("Authentication failed: %s" % variant["message"])
					socket.close()
				else:
					refresh(State.PLAYING, NetworkState.WAITING_GAMEINFO)
		elif network_state == NetworkState.WAITING_GAMEINFO:
			while socket.get_available_packet_count():
				var variant = JSON.parse_string(socket.get_packet().get_string_from_utf8())
				print("Received: [%s]" % variant)
						

func quit() -> void:
	print("Quit called")
	if server_process_state == ServerProcessState.RUNNING:
		print("Killing server")
		OS.kill(server["pid"])
		server_process_state = ServerProcessState.NOT_RUNNING
		print("Waiting threads")
		server_logs_thread.wait_to_finish()

	print("Quitting now")
	get_tree().quit()

func play_solo(play_mode) -> void:
	var _output = []
	var world_text = ""
	if play_mode == PlaySoloMode.CREATION:
		world_text = ui.world_field.get_text()
	elif play_mode == PlaySoloMode.JOIN:
		world_text = ui.worlds_tree.get_selected().get_text(0)
		
	assert(!world_text.is_empty())
	var args = ["0", "--no-input", "--instance", ProjectSettings.globalize_path("user://%s.spdb" % world_text)]
	if !OS.has_feature("release"):
		OS.set_environment("RUST_LOG", "TRACE")	
		server = OS.execute_with_pipe("../target/debug/spacebuild-server", args)
	else:
		OS.set_environment("RUST_LOG", "INFO")
		server = OS.execute_with_pipe("./spacebuild-server", args)
	if server.is_empty():
		printerr("Failed to run server")
		ui.error_placeholder.set_text("Local server not found or could not be executed")
		ui.play_button.set_disabled(false)
		return
	server_process_state = ServerProcessState.RUNNING
	server_logs_thread = Thread.new()
	server_logs_thread.start(_server_logs)
	server_uri =  "ws://localhost"
	refresh(State.WAITING_PORT, network_state)

func play_online() -> void:
	login = ui.login_field.get_text()
	if login.is_empty():
		ui.error_placeholder.set_text("Enter your login please")
		print("No login")
		ui.play_button.set_disabled(false)
		return

	var host_str = ui.host_field.get_text()
	if host_str.is_empty():
		host_str = "localhost"
		
	var port_str = ui.port_field.get_text()
	if port_str.is_empty():
		port_str = "2567"
		
	var protocol_str = "wss" if ui.encrypted_switch.is_pressed() else "ws"
	
	server_uri = "%s://%s" % [protocol_str, host_str]
	server_port = int(port_str)
	refresh(State.WAITING_PORT, network_state)
