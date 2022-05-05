# Makefile
# --------
# makefile for development, rules:
#  + `animation` - create simple gif animation, useful to create
#                  rustracer usage examples (`ffmpeg` needed)
#  + `docs`      - preview `rustracer` documentation locally
#  + `ccov`      - preview `rustracer` code-coverage html report locally

animation: examples/demo

examples/demo:
	@printf "[info] create demo gif animation ... "
	@mkdir -p $@/frames
	@for a in `seq 0 359`; do \
		rustracer demo --width 320 --height 240 --angle-deg $$a $@/frames/frame-`printf '%03d' $$a`.png; done;
	@ffmpeg -loglevel panic -f image2 -framerate 25 -i $@/frames/frame-%03d.png $@/demo.gif
	@printf "done\n"

docs: docs.pid
ccov: ccov.pid

docs.pid:
	@for f in `ls -1 target/doc/rustracer`; do ln -sf rustracer/$$f target/doc/$$f; done;
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
	@pkill -f "python3 -m http.server -b 127.0.0.1 -d target/"
	@rm $^
	@printf "done\n"
