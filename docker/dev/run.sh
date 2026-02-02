#!/bin/bash
(cd backend && cargo watch -x "run") &
(cd menu_front && trunk watch) &
(cd test_front && trunk watch)