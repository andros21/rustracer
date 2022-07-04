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
		rustracer demo --width 500 --height 375 --anti-aliasing 3 -f 1 --angle-deg $$a \
		examples/demo/frames/frame-`printf '%03d' $$a`.png; done;
	@ffmpeg -loglevel panic -f image2 -framerate 10 \
		-i examples/demo/frames/frame-%03d.png examples/demo.gif
	@rm -fr examples/demo
	@printf "done\n"

docs: patch_docs docs.pid
ccov: ccov.pid

patch_docs:
	@for f in `ls -1 target/doc/rustracer`; do ln -sf rustracer/$$f target/doc/$$f; done;
	@cp install.sh target/doc/

docs.pid:
	@printf "starting http.server ... "
	@{ python3 -m http.server -b 127.0.0.1 -d target/doc/ 8080 >/dev/null 2>&1 \
		& echo $$! >$@; }
	@printf "done\n"
	@printf "docs url: http://localhost:8080/\n"

ccov.pid:
	@printf "starting http.server ... "
	@{ python3 -m http.server -b 127.0.0.1 -d target/llvm-cov/html 8081 >/dev/null 2>&1 \
		& echo $$! >$@; }
	@printf "done\n"
	@printf "ccov url: http://localhost:8081/\n"

stop: *.pid
	@printf "stopping http.server ... "
	@pkill -F docs.pid 2>/dev/null || exit 0
	@pkill -F ccov.pid 2>/dev/null || exit 0
	@rm $^
	@printf "done\n"
