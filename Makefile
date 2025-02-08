png:
	cargo run --release
	convert test.ppm test.png

debug-png:
	DEBUG_SAMPLING=1 cargo run --release
	convert test.ppm test.png

watch:
	echo scene.toml | entr -ac make png

scene:
	./target/release/raymart $(SCENE)
	convert test.ppm test.png

debug-scene:
	DEBUG_SAMPLING=1 ./target/release/raymart $(SCENE)
	convert test.ppm test.png
