<div align="center">
  🛠️🦴
</div>
<h1 align="center">
  splint
</h1>

<div align="center">
  ensures structures with a well defined shape stay in place
</div>

<div align="center">
  <a href="https://github.com/softprops/splint/actions">
		<img src="https://github.com/softprops/splint/workflows/Main/badge.svg"/>
	</a>
</div>

<br />

## about

Splint is intended to be a continuous integration tool for your instrustructure as code. It uses json schemas to validate your
static json and yaml files. Most infrastruture files have a well defined format. This format can typically be expressed by a [JSON schema](https://json-schema.org/). Splint leverages [JSON Schema store](http://schemastore.org/json/) by default to resolve a schema that applies to your files based on their name. You may also provide your own schema for your custom infrastucture files.

Doug Tangren (softprops) 2019