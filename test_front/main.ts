// specify used components

import * as _socket from 'commons/socket'
import * as _list from 'commons/list'
import * as _client_info from 'commons/client_info'
import * as _canvas from './ts/canvas'
import * as _paste from './ts/paste'
import * as _mouse from './ts/mouse'

declare var socket;
socket = _socket;

declare var list;
list = _list;

declare var client_info;
client_info = _client_info;

declare var canvas;
canvas = _canvas;

declare var paste;
paste = _paste;

declare var mouse;
mouse = _mouse;