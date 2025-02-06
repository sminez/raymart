png:
	cargo run --release > test.ppm
	convert test.ppm test.png

debug-png:
	DEBUG_SAMPLING=1 cargo run --release > test.ppm
	convert test.ppm test.png

watch:
	echo scene.json | entr -ac make png

scene:
	./target/release/raymart $(SCENE) > test.ppm
	convert test.ppm test.png

debug-scene:
	DEBUG_SAMPLING=1 ./target/release/raymart $(SCENE) > test.ppm
	convert test.ppm test.png
