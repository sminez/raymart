png:
	cargo run --release > test.ppm
	convert test.ppm test.png
