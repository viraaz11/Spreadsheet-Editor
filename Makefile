all: build

build:		# FOR AUTOGRADER
	cargo build --release --package spreadsheet

ext1:		# EXTENSIONS PART
	cargo run --package ext -- 999 18278

test:
	cargo test --package spreadsheet

coverage:
	cargo tarpaulin --exclude-files cli/src/main.rs ext/src/*

docs: cargo-docs
	
cargo-docs:
	cargo doc --package spreadsheet

# report/report.pdf: report/report.tex
# 	pdflatex -output-directory=report report/report.tex
# 	bibtex report/report || true
# 	pdflatex -output-directory=report report/report.tex
# 	pdflatex -output-directory=report report/report.tex

.PHONY: all build ext1 test coverage docs
