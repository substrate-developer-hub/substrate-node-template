#!/usr/bin/env bash

name=$1
shift
author=$1
shift

if [[ "$name" == "" || "$name" == "-"* ]]
then
  echo "Usage: substrate-node-rename <NAME> <AUTHOR>"
  exit 1
fi
if [[ "$author" == "" || "$author" == "-"* ]]
then
  echo "Usage: substrate-node-rename <NAME> <AUTHOR>"
  exit 1
fi

lname="$(echo $name | tr '[:upper:]' '[:lower:]')"
dirname="${lname// /-}"

bold=$(tput bold)
normal=$(tput sgr0)

if [ -d "$dirname" ]; then
  echo "Directory '$name' already exists!"
  exit 1
fi

echo "${bold}Moving project folder...${normal}"

git mv substrate-node-template $dirname

pushd $dirname >/dev/null

echo "${bold}Customizing project...${normal}"
function replace {
	find_this="$1"
	shift
	replace_with="$1"
	shift
	IFS=$'\n'
	TEMP=$(mktemp -d "${TMPDIR:-/tmp}/.XXXXXXXXXXXX")
	rmdir $TEMP
	for item in `find . -not -path '*/\.*' -type f \( -name "*.rs" -o -name "*.md" -o -name "build.sh" -o -name "Cargo.toml" -o -name "Cargo.lock" \)`
	do
		sed "s/$find_this/$replace_with/g" "$item" > $TEMP
		cat $TEMP > "$item"
	done
	rm -f $TEMP
}

replace "Template Node" "${name}"
replace node-template "${lname//[_ ]/-}"
replace node_template "${lname//[- ]/_}"
replace Anonymous "$author"

echo "Rename Complete"

popd >/dev/null

