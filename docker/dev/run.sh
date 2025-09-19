#!/bin/bash
echo 'what' &&
(cd backend && cargo watch -x "run") &
(cd menu_front && trunk watch)