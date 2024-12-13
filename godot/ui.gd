extends Control

enum WelcomeState {SOLO, ONLINE}

var welcome_state = WelcomeState.SOLO
var root = null
var selected_world = null

@onready var core = get_tree().get_first_node_in_group("core")
@onready var worlds_tree = get_tree().get_first_node_in_group("worlds_tree")
@onready var modale = get_tree().get_first_node_in_group("modale")
@onready var solo_tab = get_tree().get_first_node_in_group("solo_tab")
@onready var login_field = get_tree().get_first_node_in_group("login_field")
@onready var play_button = get_tree().get_first_node_in_group("play_button")
@onready var quit_button = get_tree().get_first_node_in_group("quit_button")
@onready var world_field = get_tree().get_first_node_in_group("world_field")
@onready var create_button = get_tree().get_first_node_in_group("create_button")
@onready var gamemode_tabs = get_tree().get_first_node_in_group("gamemode_tabs")
@onready var background = get_tree().get_first_node_in_group("background")
@onready var encrypted_switch = get_tree().get_first_node_in_group("encrypted_switch")
@onready var error_placeholder = get_tree().get_first_node_in_group("error_placeholder")
@onready var host_field = get_tree().get_first_node_in_group("host_field")
@onready var port_field = get_tree().get_first_node_in_group("port_field")
@onready var screen_size = get_viewport().get_visible_rect().size

func _ready() -> void:
	root = worlds_tree.create_item()
	worlds_tree.hide_root = true
	
	get_tree().get_root().size_changed.connect(_on_size_changed)

	login_field.text_changed.connect(_on_login_changed)
	world_field.text_changed.connect(_on_world_changed)
	worlds_tree.item_selected.connect(_worlds_item_selected)
	worlds_tree.nothing_selected.connect(_worlds_nothing_selected)
	create_button.pressed.connect(_create_button_pressed)
	quit_button.pressed.connect(_quit_pressed)
	gamemode_tabs.tab_changed.connect(_gamemode_changed)
	play_button.pressed.connect(_play_button_pressed)
	encrypted_switch.toggled.connect(_on_encrypted_switch_toggled)

	_on_encrypted_switch_toggled(false)
	_on_size_changed()
	refresh(welcome_state)
	
	if OS.has_feature("web"):
		gamemode_tabs.remove_child(solo_tab)

func _on_encrypted_switch_toggled(toggled):
	if toggled:
		encrypted_switch.set_text("on")
	else:
		encrypted_switch.set_text("off")

func refresh(dest_ui_welcome_state) -> void:
	if core.state == core.State.WELCOME:
		if core.state != core.State.WELCOME:
			pass
		else:
			if dest_ui_welcome_state == WelcomeState.SOLO:
				if welcome_state != WelcomeState.SOLO:
					list_worlds()
					play_button.set_disabled(true)
				else:
					play_button.set_disabled(selected_world == null)
					create_button.set_disabled(world_field.get_text().is_empty())
			elif dest_ui_welcome_state == WelcomeState.ONLINE:
				play_button.set_disabled(login_field.get_text().is_empty())
	elif core.state == core.State.INIT:
		if dest_ui_welcome_state == WelcomeState.SOLO:
				list_worlds()
				play_button.set_disabled(true)
	welcome_state = dest_ui_welcome_state

func _create_button_pressed():
	core.play_solo(core.PlaySoloMode.CREATION)

func _play_button_pressed():
	if welcome_state == WelcomeState.ONLINE:
		core.play_online()
	elif welcome_state == WelcomeState.SOLO:
		core.play_solo(core.PlaySoloMode.JOIN)
	else:
		assert(false)

func _quit_pressed() -> void:
	core.quit()

func _gamemode_changed(tab_id):
	var dest_welcome_state
	if tab_id == 1:
		dest_welcome_state = WelcomeState.ONLINE
	elif tab_id == 0:
		dest_welcome_state = WelcomeState.SOLO
	else:
		assert(false)
	refresh(dest_welcome_state)

func _worlds_item_selected():
	selected_world = worlds_tree.get_selected()
	refresh(welcome_state)

func _worlds_nothing_selected():
	refresh(welcome_state)

func _on_login_changed(_text):
	refresh(welcome_state)

func _on_world_changed(_text):
	refresh(welcome_state)

func _on_size_changed():
	var new_screen_size = get_viewport().get_visible_rect().size
	#var ref = Vector2(1920, 1080)
	var ref = screen_size
	set_scale((new_screen_size / ref).clamp(Vector2.ONE * 0.1, Vector2.ONE * 1000))
	screen_size = new_screen_size
	
func list_worlds():
	var dir = DirAccess.open("user://")
	var files = dir.get_files()
	worlds_tree.clear()
	root = worlds_tree.create_item()
	for file in files:
		var item = worlds_tree.create_item(root) as TreeItem
		item.set_text(0, file.trim_suffix(".spdb"))

func _process(_delta: float) -> void:
	pass
