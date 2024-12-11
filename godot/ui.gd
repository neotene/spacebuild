extends Node

enum WelcomeState {SOLO, ONLINE}

var welcome_state = WelcomeState.SOLO
var root = null
var selected_world = null

@onready var core = $"../../Core"
@onready var worlds_tree = $"Modale/Welcome/GameMode/Solo/Worlds"
@onready var ratio = $"Modale"
@onready var solo_tab = $'Modale/Welcome/GameMode/Solo'
@onready var login_field = $"Modale/Welcome/GameMode/Online/Login/LineEdit"
@onready var play_button = $"Modale/Welcome/Actions/Play"
@onready var quit_button = $"Modale/Welcome/Actions/Quit"
@onready var world_field = $"Modale/Welcome/GameMode/Solo/WorldCreation/LineEdit"
@onready var create_button = $"Modale/Welcome/GameMode/Solo/WorldCreation/Create"
@onready var gamemode_tabs = $"Modale/Welcome/GameMode"
@onready var background = $"Modale/Background"
@onready var encrypted_switch = $"Modale/Welcome/GameMode/Online/Encrypted/CheckButton"
@onready var error_placeholder = $"Modale/Welcome/Actions/ErrorPlaceholder"
@onready var host_field = $"Modale/Welcome/GameMode/Online/Host/LineEdit"
@onready var port_field = $"Modale/Welcome/GameMode/Online/Port/LineEdit"

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

	_on_size_changed()
	refresh(welcome_state)
	
	if OS.has_feature("web"):
		gamemode_tabs.remove_child(solo_tab)

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
	core.play(core.PlayMode.SOLO_CREATION)

func _play_button_pressed():
	if welcome_state == WelcomeState.ONLINE:
		core.play(core.PlayMode.ONLINE)
	elif welcome_state == WelcomeState.SOLO:
		core.play(core.PlayMode.SOLO_JOIN)
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
	var screen_size = get_viewport().get_visible_rect().size
	ratio.set_ratio(screen_size.x / screen_size.y)
	
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
