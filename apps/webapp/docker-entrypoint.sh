#!/bin/sh

set -eu

HTML_DIR=/usr/share/nginx/html
SRC_DIR=/usr/local/src/oxid
CONFIG_FILE="$HTML_DIR/config.json"

rm -rf -- "${HTML_DIR:?}/"* "${HTML_DIR:?}/".[!.]* "${HTML_DIR:?}/"..?*
cp -r "$SRC_DIR"/* "$HTML_DIR"

# Every value the client needs at runtime is templated here. The OIDC client id
# used to be a build-time variable that nothing supplied, so the published image
# could never authenticate: the config mechanism answered "where" but not "who".
substitute() {
  placeholder="$1"
  value="$2"
  escaped=$(printf '%s' "$value" | sed -e 's/[\/&|]/\\&/g')
  # shellcheck disable=SC2016
  sed -i "s|\${$placeholder}|$escaped|g" "$CONFIG_FILE"
}

if [ -f "$CONFIG_FILE" ]; then
  substitute API_URL "${API_URL:-}"
  substitute ISSUER_URL "${ISSUER_URL:-}"
  substitute OIDC_CLIENT_ID "${OIDC_CLIENT_ID:-}"
  substitute OIDC_SCOPE "${OIDC_SCOPE:-openid profile email}"
fi

exec "$@"
