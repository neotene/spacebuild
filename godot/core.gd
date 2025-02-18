extends Node

enum State {INIT, WELCOME, WAITING_PORT, LOADING, PLAYING_SOLO, PLAYING_ONLINE, LEAVING, STOPPING_GAME, QUITTING}
enum NetworkState {IDLE, CONNECTING, AUTHENTICATING, WAITING_GAMEINFO, CLOSING}
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
var close_timer = 0

var to_instantiate = []
var instantiate_timer = 0
var instantiate_limit = 0.01

var bodies_infos = {}

@onready var asteroid_scene = load("res://scenes/asteroid.tscn")
@onready var planet_scene = load("res://scenes/planet.tscn")
@onready var moon_scene = load("res://scenes/moon.tscn")
@onready var star_scene = load("res://scenes/star.tscn")
@onready var player_scene = load("res://scenes/player.tscn")

@onready var server_logs = get_tree().get_first_node_in_group("server_logs")
@onready var container = get_tree().get_first_node_in_group("container") as Node3D
@onready var player = get_tree().get_first_node_in_group("player")

@onready var ui = get_tree().get_first_node_in_group("ui")
var regex = RegEx.new()

@onready var info = get_tree().get_first_node_in_group("info")

func _notification(what):
	if what == NOTIFICATION_WM_CLOSE_REQUEST:
		quit()

func _ready() -> void:
	regex.compile("^.*Server loop starts, listenning on (\\d+)$")
	get_tree().set_auto_accept_quit(false)

func _server_logs(key):
	var pipe = server[key] as FileAccess
	var line_empty_cnt = 0

	while server_process_state == ServerProcessState.RUNNING:
		var line = pipe.get_line()
		if pipe.eof_reached():
			break
			
		if line.is_empty():
			line_empty_cnt += 1
			if line_empty_cnt == 10:
				print("Got %d empty lines, reader thread (%s) quitting now..." % [line_empty_cnt, key])
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
		if !line.is_empty():
			print("> %s" % line)
		if !OS.has_feature("release"):
			server_logs.call_deferred("append_text", line)
			server_logs.call_deferred("newline")
	
	print("Server log (%s) thread quitting now!" % key)

func refresh(to_state, to_network_state) -> void:
	if state != State.WELCOME && to_state == State.WELCOME:
		container.remove_from_group("celestial")
		bodies_infos.clear()

	get_tree().get_first_node_in_group("modale").set_visible(to_state == State.WELCOME)
	get_tree().get_first_node_in_group("title").set_visible(to_state == State.WELCOME)
	get_tree().get_first_node_in_group("loading").set_visible(to_state == State.WAITING_PORT
								 || to_state == State.LOADING)
	if to_state == State.WAITING_PORT:
		get_tree().get_first_node_in_group("loading").set_text("Waiting server...")
	elif to_state == State.LOADING:
		get_tree().get_first_node_in_group("loading").set_text("Connecting...")

	get_tree().get_first_node_in_group("ship").set_visible(to_state == State.PLAYING_SOLO || to_state == State.PLAYING_ONLINE)
	get_tree().get_first_node_in_group("container").set_visible(to_state == State.PLAYING_SOLO || to_state == State.PLAYING_ONLINE)

	ui.refresh(to_state, ui.welcome_state)
	state = to_state
	network_state = to_network_state
	
func _process(delta: float) -> void:
	if state == State.INIT:
		refresh(State.WELCOME, network_state)
		return


	if state == State.PLAYING_SOLO || state == State.PLAYING_ONLINE:
		for key in bodies_infos:
			var body = container.get_node_or_null(str(key)) as Node3D
			assert(body)
			var body_info = bodies_infos[key]
			if !bodies_infos.has(body_info.gravity_center):
				continue
			var gravity_center = container.get_node_or_null(str(body_info.gravity_center))
			assert(gravity_center)
			var body_transform = body.transform as Transform3D
			#body_transform = body_transform.translated(-gravity_center.global_position)
			#body_transform = body_transform.rotated(Vector3.UP, body_info.rotating_speed / 2 * delta)
			#body_transform = body_transform.translated(gravity_center.global_position)
			#body.transform = body_transform
			
			

	if state != State.PLAYING_SOLO && state != State.PLAYING_ONLINE:
		instantiate_timer = 0
	else:
		instantiate_timer += delta
		if instantiate_timer < instantiate_limit || to_instantiate.is_empty():
			info.set_visible(false)
		else:
			info.set_visible(true)
			for galactic_to_instantiate in to_instantiate:
				#print("Instantiating %s" % galactic_to_instantiate)
				var color = Color()
				var galactic_tree
				if galactic_to_instantiate.type == "Asteroid":
					galactic_tree = asteroid_scene.instantiate()
					color = Color(1, 0, 0)
				elif galactic_to_instantiate.type == "Planet":
					galactic_tree = planet_scene.instantiate()
					color = Color(0, 0, 1)
				elif galactic_to_instantiate.type == "Moon":
					galactic_tree = moon_scene.instantiate()
					color = Color(0, 1, 1)
				elif galactic_to_instantiate.type == "Star":
					galactic_tree = star_scene.instantiate()
					color = Color(1, 1, 1)
				elif galactic_to_instantiate.type == "Player":
					galactic_tree = player_scene.instantiate()
					color = Color(0, 1, 0)
				else:
					assert(false)
				var model = galactic_tree.get_child(0)
				(model.material as StandardMaterial3D).albedo_color = color
				galactic_tree.position = galactic_to_instantiate.coords
				galactic_tree.set_name(str(int(galactic_to_instantiate.id)))
				container.add_child(galactic_tree)
				bodies_infos[int(galactic_to_instantiate.id)] = {
					"gravity_center": galactic_to_instantiate.gravity_center,
					"rotating_speed": galactic_to_instantiate.rotating_speed,
				}
				instantiate_timer -= instantiate_limit
				if instantiate_timer < instantiate_limit:
					break
			to_instantiate.clear()

	handle_server(delta)
	
	handle_network(delta)

func connect_to_server():
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

func handle_server(delta):
	if state == State.WAITING_PORT:
		mutex.lock()
		var port = server_port
		mutex.unlock()
		
		if !port:
			return
		
		server_uri += ":%d" % port
		print("Connecting to %s..." % server_uri)
		connect_to_server()

	if state == State.STOPPING_GAME || state == State.QUITTING:
		stop_timer += delta;
		
		if !OS.is_process_running(server["pid"]):
			if state == State.QUITTING:
				quit_now(true);
			else:
				server = Dictionary()
				server_process_state = ServerProcessState.NOT_RUNNING
				refresh(State.WELCOME, network_state)
				stop_timer = 0
				return
				
		if stop_timer < 10:
			return ;
			
		stop_timer = 0

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

func handle_network(delta):
	var new_state = state
	var new_network_state = network_state

	if network_state != NetworkState.IDLE:
		socket.poll()

		var socket_state = socket.get_ready_state()
		if socket_state == WebSocketPeer.STATE_CLOSING:
			if state == State.LEAVING:
				if close_timer == 0:
					print("Closing")
					new_network_state = NetworkState.CLOSING
				if close_timer >= 0.5:
					socket = WebSocketPeer.new()
					new_network_state = NetworkState.IDLE
					if ui.welcome_state == ui.WelcomeState.SOLO && !OS.has_feature("web"):
						stop_server()
					else:
						new_state = State.WELCOME
					close_timer = 0
				else:
					close_timer += delta
					
	
		elif socket_state == WebSocketPeer.STATE_CLOSED:
			var code = socket.get_close_code()
			var reason = socket.get_close_reason()
			print("WebSocket closed with code: %d, reason %s. Clean: %s" % [code, reason, code != -1])
			socket = WebSocketPeer.new()
			new_network_state = NetworkState.IDLE
			if state == State.PLAYING_SOLO:
				stop_server()
			else:
				new_state = State.WELCOME
			close_timer = 0
				

		elif network_state == NetworkState.CONNECTING && socket_state == WebSocketPeer.STATE_OPEN:
			if ui.welcome_state == ui.WelcomeState.ONLINE:
				login_hash["Login"]["nickname"] = ui.login_field.get_text()
			else:
				login_hash["Login"]["nickname"] = "Player"
			if socket.send_text(JSON.stringify(login_hash)) != OK:
				print("Send error")
			else:
				new_network_state = NetworkState.AUTHENTICATING

		elif network_state == NetworkState.AUTHENTICATING:
			if socket.get_available_packet_count():
				var variant = JSON.parse_string(socket.get_packet().get_string_from_utf8())
				if variant["success"] == false:
					print("Login failure: %s" % variant["message"])
					ui.error_placeholder.set_text("Authentication failed: %s" % variant["message"])
					leave()
				else:
					print("Login success, id is %s" % variant["message"])
					new_network_state = NetworkState.WAITING_GAMEINFO
					if server_process_state == ServerProcessState.RUNNING:
						new_state = State.PLAYING_SOLO
					else:
						new_state = State.PLAYING_ONLINE

		elif network_state == NetworkState.WAITING_GAMEINFO:
			while socket.get_available_packet_count():
				var variant = JSON.parse_string(socket.get_packet().get_string_from_utf8())
				#print("Received: %s" % variant)
				#var galactics = container.get_children()
				if variant.has("Player"):
					var coords = variant["Player"]["coords"]
					#print(coords)
					player.position = Vector3(coords[0], coords[1], coords[2])
					
				#elif variant.has("PlayersInSystem"):
					#var elements = variant["PlayersInSystem"] as Array
					#
					#for galactic in galactics:
						#var found = false
						#for element in elements:
							#if element["id"] == galactic.get_name():
								#found = true
								#break
						#if !found:
							#container.remove_child(galactic)
							#
					#
					#for element in elements:
						#var found = false
						#for galactic in galactics:
							#if galactic.get_name() == element["id"]:
								#galactic.position = Vector3(element["coords"][0], element["coords"][1], element["coords"][2])
								#found = true
								#break
						#if !found:
							#to_instantiate.push_back({
								#"id": element["id"],
								#"type": "Player",
								#"coords": Vector3(element["coords"][0], element["coords"][1], element["coords"][2])})

				elif variant.has("BodiesInSystem"):
					var elements = variant["BodiesInSystem"] as Array
					
					for element in elements:
						var galactic = container.get_node_or_null(str(int(element["id"])))
						if galactic:
							galactic.position = Vector3(element["coords"][0], element["coords"][1], element["coords"][2])
						else:
							to_instantiate.push_back({
								"id": int(element["id"]),
								"type": element["element_type"],
								"coords": Vector3(element["coords"][0], element["coords"][1], element["coords"][2]),
								"gravity_center": int(element["gravity_center"]),
								"rotating_speed": element["rotating_speed"],
								})

	refresh(new_state, new_network_state)
			
func quit_now(wait_threads):
	if wait_threads:
		print("Waiting threads")
		server_logs_err_thread.wait_to_finish()
		server_logs_out_thread.wait_to_finish()
	get_tree().quit()

func leave():
	if state == State.LEAVING || state == State.STOPPING_GAME:
		return
	assert(state != State.QUITTING && state != State.LOADING)
	print("Leaving...")
	socket.close()
	refresh(State.LEAVING, network_state)

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
	if !OS.has_feature("release"):
		var manifest_path = ProjectSettings.globalize_path("res://../Cargo.toml")
		#OS.set_environment("RUST_LOG", "TRACE")
		var ret
		if OS.has_feature("windows"):
			ret = OS.execute("powershell.exe", ["-Command", "Get-Process -Name spacebuild-server | Stop-Process"])
		else:
			ret = OS.execute("bash", ["-c", "killall spacebuild-server"])

		if ret == -1:
			print("Cleaner failed to execute")
		elif ret == 0:
			print("Had to clean remaining server instances")

		var args = ["run", "--manifest-path", manifest_path, "--bin", "spacebuild-server", "--", "0",
			"--instance", ProjectSettings.globalize_path("user://%s.sbdb" % world_text), "--trace-level", "INFO"]
		server = OS.execute_with_pipe("cargo", args)
	else:
		#OS.set_environment("RUST_LOG", "INFO")
		var args = ["0", "--instance", ProjectSettings.globalize_path("user://%s.sbdb" % world_text), "--trace-level", "INFO"]
		server = OS.execute_with_pipe("./spacebuild-server", args)
	if server.is_empty():
		printerr("Failed to run server")
		ui.error_placeholder.set_text("Local server not found or could not be executed")
		ui.play_button.set_disabled(false)
		return

	server_process_state = ServerProcessState.RUNNING
	server_port = 0
	if server_logs_err_thread && server_logs_err_thread.is_started():
		server_logs_err_thread.wait_to_finish()
	server_logs_err_thread = Thread.new()
	server_logs_err_thread.start(_server_logs.bind("stderr"))
	if server_logs_out_thread && server_logs_out_thread.is_started():
		server_logs_out_thread.wait_to_finish()
	server_logs_out_thread = Thread.new()
	server_logs_out_thread.start(_server_logs.bind("stdio"))
	server_uri = "ws://localhost"
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

	connect_to_server()
