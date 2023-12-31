# makefile
# --------
# makefile for development, rules:
#  + `demo.gif` - create demo scene gif animation (`ffmpeg` needed)
#  + `docs`     - preview `rustracer` documentation locally
#  + `ccov`     - preview `rustracer` code-coverage html report locally

demo.gif: examples/demo.gif

examples/demo.gif:
	@printf "[info] create demo gif animation ... "
	@mkdir -p examples/demo/frames
	@for a in `seq 0 359`; do \
		rustracer demo --width 500 --height 375 --anti-aliasing 3 --angle-deg $$a \
		examples/demo/frames/frame-`printf '%03d' $$a`.png; done;
	@ffmpeg -loglevel panic -f image2 -framerate 10 \
		-i examples/demo/frames/frame-%03d.png examples/demo.gif
	@rm -fr examples/demo
	@printf "done\n"

docs: rust_docs docs.pid
ccov: rust_ccov ccov.pid

rust_docs:
	@mv README.md README.orig.md
	@grep -v "<hr" README.orig.md > README.md
	@sed -i 's/<h5>/<h6>/;s/<\/h5>/<\/h6>/' README.md
	@sed -zi 's/<\/h6>/<\/h6><br>/2;s/<\/h6>/<\/h6><br>/3' README.md
	@sed -zi 's/<\/h6>/<\/h6><br>/5' README.md
	@sed -zi 's/<\/h6>/<\/h6><br>/6' README.md
	@sed -i 's/<h4>/<h5>/;s/<\/h4>/<\/h5>/' README.md
	@cargo rustdoc --locked
	@rm -f README.md
	@mv README.orig.md README.md
	@mv target/doc/rustracer/ target/doc/docs
	@find target/doc/ -name "*.html" -exec sed -i 's/\.\.\/rustracer\//\.\.\/docs\//g' {} \;
	@sed -i 's/item.href/item.href.replace("rustracer","docs")/' target/doc/static.files/search-*js
	@cp install.sh target/doc/

docs.pid:
	@printf "starting http.server ... "
	@{ python3 -m http.server -b 127.0.0.1 -d target/doc 8080 >/dev/null 2>&1 \
		& echo $$! >$@; }
	@printf "done\n"
	@printf "docs url: http://localhost:8080/docs\n"

rust_ccov:
	@cargo tarpaulin --tests --locked --output-dir target/tarpaulin/ --out html --exclude-files src/cli.rs src/main.rs

ccov.pid:
	@printf "starting http.server ... "
	@ln -sf ./tarpaulin-report.html target/tarpaulin/index.html
	@{ python3 -m http.server -b 127.0.0.1 -d target/tarpaulin/ 8081 >/dev/null 2>&1 \
		& echo $$! >$@; }
	@printf "done\n"
	@printf "ccov url: http://localhost:8081/\n"

stop: *.pid
	@printf "stopping http.server ... "
	@pkill -F docs.pid 2>/dev/null || exit 0
	@pkill -F ccov.pid 2>/dev/null || exit 0
	@rm $^
	@printf "done\n"
