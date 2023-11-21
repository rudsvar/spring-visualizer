.PHONY: demo/example.dot

demo/example.png: demo/example.dot
	dot -Tpng $< > $@

demo/example.dot:
	spring-visualizer com/example/demo > $@
