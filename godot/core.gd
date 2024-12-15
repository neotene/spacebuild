extends Node

enum State {INIT, WELCOME, WAITING_PORT, LOADING, PLAYING_SOLO, PLAYING_ONLINE, STOPPING_GAME, QUITTING}
enum NetworkState {IDLE, CONNECTING, AUTHENTICATING, WAITING_GAMEINFO}
enum ServerProcessState {NOT_RUNNING, RUNNING}
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
var server_logs_out_thread: Thread
var server_logs_err_thread: Thread
var mutex: Mutex = Mutex.new()
var server_port = 0
var server_uri: String = ""
var stop_timer = 0

@onready var server_logs = get_tree().get_first_node_in_group("server_logs")
@onready var container = get_tree().get_first_node_in_group("container")

@onready var ui = get_tree().get_first_node_in_group("ui")
var regex = RegEx.new()

func _notification(what):
	if what == NOTIFICATION_WM_CLOSE_REQUEST:
		quit()

func _ready() -> void:
	regex.compile("^.*Server loop starts, listenning on (\\d+)$")
	get_tree().set_auto_accept_quit(false)

func _server_logs(key):
	var pipe = server[key] as FileAccess

	while server_process_state == ServerProcessState.RUNNING:	
		var line = pipe.get_line()
		if pipe.eof_reached() || line.is_empty():
			break
		if key == "stderr":
			var search_result = regex.search(line)
			if search_result:
				var port_str = search_result.get_string(1)
				assert(!port_str.is_empty())
				print("Found port: %s" % port_str)
				mutex.lock()
				server_port = int(port_str)
				mutex.unlock()
		print("Server says on %s: [%s]" % [key, line])
		if !OS.has_feature("release"):
			server_logs.call_deferred("append_text", line)
			server_logs.call_deferred("newline")
	
	print("Server log (%s) thread quitting now!" % key)

func refresh(to_state, to_network_state) -> void:
	#ui.error_placeholder.set_text("Connecting...")
	get_tree().get_first_node_in_group("modale").set_visible(to_state == State.WELCOME)
	get_tree().get_first_node_in_group("title").set_visible(to_state == State.WELCOME)
	get_tree().get_first_node_in_group("loading").set_visible(to_state == State.WAITING_PORT
								 || to_state == State.LOADING)
	if to_state == State.WAITING_PORT:
		get_tree().get_first_node_in_group("loading").set_text("Waiting server...")
	elif to_state == State.LOADING:
		get_tree().get_first_node_in_group("loading").set_text("Connecting...")

	ui.refresh(to_state, ui.welcome_state)
	state = to_state
	network_state = to_network_state
	
func _process(delta: float) -> void:
	if state == State.INIT:
		state = State.WELCOME
		return
		
	handle_server(delta)
	
	handle_network()

func handle_server(delta):
	if state == State.WAITING_PORT:
		mutex.lock()
		var port = server_port
		mutex.unlock()
		
		if !port:
			return 
		
		server_uri += ":%d" % port
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

	if state == State.STOPPING_GAME || state == State.QUITTING:
		stop_timer += delta;
		
		if !OS.is_process_running(server["pid"]):
			if state == State.QUITTING:
				quit_now(true);
			else:
				server = Dictionary()
				server_process_state = ServerProcessState.NOT_RUNNING
				refresh(State.WELCOME, network_state)
				return 
				
		if stop_timer < 5:
			return ;
			
		print("Killing server!")
		OS.kill(server["pid"])
		server_process_state = ServerProcessState.NOT_RUNNING

		if state == State.QUITTING:
			quit_now(true)
			
		server = Dictionary()
		refresh(State.WELCOME, network_state)
		return
			
	if server_process_state == ServerProcessState.RUNNING:
		if !server.is_empty() && !OS.is_process_running(server["pid"]):
			server_process_state = ServerProcessState.NOT_RUNNING
			refresh(State.WELCOME, network_state)
			server_logs_err_thread.wait_to_finish()
			server_logs_out_thread.wait_to_finish()

func handle_network():
	var new_state = state
	var new_network_state = network_state
	if network_state != NetworkState.IDLE:
		socket.poll()
		var socket_state = socket.get_ready_state()
		if socket_state == WebSocketPeer.STATE_CLOSING:
			socket.poll()
			print("Closing")
		elif socket_state == WebSocketPeer.STATE_CLOSED:
			var code = socket.get_close_code()
			var reason = socket.get_close_reason()
			print("WebSocket closed with code: %d, reason %s. Clean: %s" % [code, reason, code != -1])
			new_network_state = NetworkState.IDLE
			if state == State.PLAYING_SOLO:
				stop_server()
				refresh(state, new_network_state)
				return

		if network_state == NetworkState.CONNECTING:
			if socket_state == WebSocketPeer.STATE_OPEN:
				new_network_state = NetworkState.AUTHENTICATING
				login_hash["Login"]["nickname"] = ui.login_field.get_text()
				socket.send_text(JSON.stringify(login_hash))
		elif network_state == NetworkState.AUTHENTICATING:
			while socket.get_available_packet_count():
				var variant = JSON.parse_string(socket.get_packet().get_string_from_utf8())
				print("Received: %s" % variant)
				if variant["success"] == false:
					ui.error_placeholder.set_text("Authentication failed: %s" % variant["message"])
					socket.close()
				else:
					print("Login success, uuid is %s" % variant["message"])
					new_network_state = NetworkState.WAITING_GAMEINFO
					if server_process_state == ServerProcessState.RUNNING:
						new_state = State.PLAYING_SOLO
					else:
						new_state = State.PLAYING_ONLINE

		elif network_state == NetworkState.WAITING_GAMEINFO:
			while socket.get_available_packet_count():
				var variant = JSON.parse_string(socket.get_packet().get_string_from_utf8())
				var galactics = container.get_children()
				if variant.has("ElementsInSystem"):
					var elements = variant["ElementsInSystem"] as Array
					for element in elements:
						var found = false
						for galactic in galactics:
							if galactic.get_name() == element["uuid"]:
								galactic.position = Vector3(element["coords"][0], element["coords"][1], element["coords"][2])
								found = true
								break
						if !found:
							var galactic = preload("res://galactic.tscn")
							var node = galactic.instantiate();
							node.set_name(element["uuid"])
							container.add_child(node)
	refresh(new_state, new_network_state)
			
func quit_now(wait_threads):
	if wait_threads:
		print("Waiting threads")
		server_logs_err_thread.wait_to_finish()
		server_logs_out_thread.wait_to_finish()
	get_tree().quit()

func leave():
	socket.close(0)
	container.remove_from_group("galactic")

func stop_server():
	print("Stopping server gracefully...")
	(server["stdio"] as FileAccess).store_line("stop")
	(server["stdio"] as FileAccess).flush()
	state = State.STOPPING_GAME

func quit() -> void:
	print("Quit called")
	if server_process_state == ServerProcessState.RUNNING:
		stop_server()
		state = State.QUITTING
	else:
		print("Server not running, quitting now!")
		quit_now(false)

func play_solo(play_mode) -> void:
	var _output = []
	var world_text = ""
	if play_mode == PlaySoloMode.CREATION:
		world_text = ui.world_field.get_text()
	elif play_mode == PlaySoloMode.JOIN:
		world_text = ui.worlds_tree.get_selected().get_text(0)
		
	assert(!world_text.is_empty())
	var args = ["0", "--instance", ProjectSettings.globalize_path("user://%s.sbdb" % world_text), "--trace-level", "INFO"]
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
	if server_logs_err_thread:
		server_logs_err_thread.wait_to_finish()
	server_logs_err_thread = Thread.new()
	server_logs_err_thread.start(_server_logs.bind("stderr"))
	if server_logs_out_thread:
		server_logs_out_thread.wait_to_finish()
	server_logs_out_thread = Thread.new()
	server_logs_out_thread.start(_server_logs.bind("stdio"))
	server_uri =  "ws://localhost"
	server_port = 0
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
	#refresh(State.WAITING_PORT, network_state)
	assert(false)
