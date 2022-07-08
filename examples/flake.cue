// flake.cue
// =========
// sphere-flake 3d fractal (n<=2) scene generation
//
// see https://en.wikipedia.org/wiki/Koch_snowflake variants

import "list"

import "math"

flake: {
	colors: [
		{name: "sky", color:    "[\(0/255), \(221/255), \(255/255)]"},
		{name: "ground", color: "[\(255/255), \(142/255), \(80/255)]"},
		{name: "flake", color:  "[\(95/255), \(95/255), \(95/255)]"},
	]
	materials: [
		{name: "sky", {diffuse: {uniform:    "sky"}, uniform:    "WHITE"}},
		{name: "flake", {specular: {uniform: "flake"}, uniform:  "BLACK"}},
		{name: "ground", {diffuse: {uniform: "ground"}, uniform: "BLACK"}},
	]
	transformations: [
		{name:                     "ground", compose: [{translation: "[0, 0, -1]"}]},
		{name:                     "camera", compose: [{translation: "[-2, 0, 0]"}, {rotationy:        48}]},
		{name:                     "sky", compose: [{scaling:        "[100, 100, 100]"}, {translation: "[0, 0, 0.5]"}]},
		for i, t in _flakes {name: "flake_\(i)", compose:            t},
	]
	camera: {
		type:           "\"perspective\""
		ratio:          "RATIO"
		distance:       "DISTANCE"
		transformation: "camera"
	}
	shapes: [
		{shape:                     "sphere", material: "sky", transformation:    "sky"},
		{shape:                     "plane", material:  "ground", transformation: "ground"},
		{shape:                     "sphere", material: "flake", transformation:  "IDENTITY"},
		for i, t in _flakes {shape: "sphere", material: "flake", transformation:  transformations[i+3].name},
	]

	// sphere-flake iterative algorithm (n<=2)
	// compose transformations for each level for each sphere
	_levels: 2
	_flakes: [
		// lower spheres cross lower spheres
		for i in list.Range(1, _levels+1, 1)
		for j in list.Range(0, math.Pow(6, i-1), 1)
		for k in list.Range(0, 6, 1) {[
			{scaling:                                  "[\(1/math.Pow(3, i)), \(1/math.Pow(3, i)), \(1/math.Pow(3, i))]"},
			{translation:                              "[\(1/math.Pow(3, i-1)+1/math.Pow(3, i)), 0, 0]"},
			{rotationz:                                60 * k},
			{rotationy:                                90 * (i - 1)},
			for l in list.Range(1, i, 1) {translation: "[\((1/math.Pow(3, l-1)+1/math.Pow(3, l))*math.Sin(math.Pi*0.5*l)), 0, \((1/math.Pow(3, l-1)+1/math.Pow(3, l))*math.Cos(math.Pi*0.5*l))]"},
			{rotationz:                                60 * j}]
		},
		// upper spheres cross lower spheres
		for i in list.Range(1, _levels+1, 1)
		for j in list.Range(0, math.Pow(6, i-1), 1)
		for k in list.Range(0, 3, 1) {[
			{scaling:                                  "[\(1/math.Pow(3, i)), \(1/math.Pow(3, i)), \(1/math.Pow(3, i))]"},
			{translation:                              "[\(1/math.Pow(3, i-1)+1/math.Pow(3, i)), 0, 0]"},
			{rotationy:                                -45}, {rotationz: (120 * k) + 30},
			{rotationy:                                90 * (i - 1)},
			for l in list.Range(1, i, 1) {translation: "[\((1/math.Pow(3, l-1)+1/math.Pow(3, l))*math.Sin(math.Pi*0.5*l)), 0, \((1/math.Pow(3, l-1)+1/math.Pow(3, l))*math.Cos(math.Pi*0.5*l))]"},
			{rotationz:                                60 * j}]
		},
		// lower spheres cross upper spheres
		for i in list.Range(1, _levels+1, 1)
		for j in list.Range(0, math.Pow(6, i-1)*3*(i-1), 1)
		for k in list.Range(0, 6, 1) {[
			{scaling:                                  "[\(1/math.Pow(3, i)), \(1/math.Pow(3, i)), \(1/math.Pow(3, i))]"},
			{translation:                              "[\(1/math.Pow(3, i-1)+1/math.Pow(3, i)), 0, 0]"},
			{rotationz:                                60 * k},
			{rotationy:                                90 * (i - 1)},
			for l in list.Range(1, i, 1) {translation: "[\((1/math.Pow(3, l-1)+1/math.Pow(3, l))*math.Sin(math.Pi*0.5*l)), 0, \((1/math.Pow(3, l-1)+1/math.Pow(3, l))*math.Cos(math.Pi*0.5*l))]"},
			{rotationy:                                -45}, {rotationz: (120 * j) + 30}]
		},
		// upper spheres cross upper spheres
		for i in list.Range(1, _levels+1, 1)
		for j in list.Range(0, math.Pow(6, i-1)*3*(i-1), 1)
		for k in list.Range(0, 3, 1) {[
			{scaling:                                  "[\(1/math.Pow(3, i)), \(1/math.Pow(3, i)), \(1/math.Pow(3, i))]"},
			{translation:                              "[\(1/math.Pow(3, i-1)+1/math.Pow(3, i)), 0, 0]"},
			{rotationy:                                -45}, {rotationz: (120 * k) + 30},
			{rotationy:                                90 * (i - 1)},
			for l in list.Range(1, i, 1) {translation: "[\((1/math.Pow(3, l-1)+1/math.Pow(3, l))*math.Sin(math.Pi*0.5*l)), 0, \((1/math.Pow(3, l-1)+1/math.Pow(3, l))*math.Cos(math.Pi*0.5*l))]"},
			{rotationy:                                -45}, {rotationz: (120 * j) + 30}]
		},
	]
}
