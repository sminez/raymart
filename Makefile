png:
	cargo run --release > test.ppm
	convert test.ppm test.png

scene:
	./target/release/raymart $(SCENE) > test.ppm
	convert test.ppm test.png
