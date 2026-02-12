# controls_config.py
import configparser
import os
import pygame


class ControlsConfig:
    def __init__(self, filename='config.ini'):
        self.file = filename
        self.config = configparser.ConfigParser()
        self.load()

    def load(self):
        if os.path.exists(self.file):
            self.config.read(self.file)
            if 'Bindings' not in self.config: self.config['Bindings'] = {}
            if 'Gamepad' not in self.config: self.config['Gamepad'] = {}
        else:
            self.config['Bindings'] = {
                'forward': 'key:z',
                'backward': 'key:s',
                'left': 'key:q',
                'right': 'key:d',
                'rot_left': 'key:left',
                'rot_right': 'key:right',
                'shoot': 'mouse:1',
                'shockwave': 'mouse:3',
                'sprint': 'key:lctrl',
                'slow': 'key:lalt'
            }
            self.config['Gamepad'] = {
                'shoot': 'btn:5',  # RB
                'shockwave': 'btn:4',  # LB
                'sprint': 'btn:0',  # A
                'slow': 'btn:1'  # B
            }
            self.save()

    def save(self):
        with open(self.file, 'w') as f:
            self.config.write(f)

    def get_bind(self, section, action):
        return self.config[section].get(action, '')

    def set_bind(self, section, action, value):
        if section not in self.config:
            self.config[section] = {}
        self.config[section][action] = str(value)
        self.save()

    def parse_bind(self, bind_str):
        if not bind_str:
            return None, None
        parts = bind_str.split(':')
        if len(parts) != 2:
            return None, None

        b_type, b_val = parts[0], parts[1]

        if b_type == 'key':
            try:
                return 'key', pygame.key.key_code(b_val)
            except ValueError:
                return 'key', -1
        elif b_type in ('mouse', 'btn'):
            try:
                return b_type, int(b_val)
            except ValueError:
                return b_type, -1

        return None, None

    def get_parsed_binds(self):
        parsed = {'Bindings': {}, 'Gamepad': {}}
        for section in ['Bindings', 'Gamepad']:
            for action, bind_str in self.config[section].items():
                parsed[section][action] = self.parse_bind(bind_str)
        return parsed