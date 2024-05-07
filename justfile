port := env_var_or_default('SCHMFY_PORT', '8000')

build:
  docker build -t schmfy_express .

run: build
  docker run --rm schmfy_express -p 8000:{{port}}
