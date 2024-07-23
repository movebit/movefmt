#!/bin/bash

commits=$(git log --pretty=format:"* [[\`%h\`](https://github.com/movebit/movefmt/commit/%H)] - %s (%an)")

echo -e "### Commits\n" > commits.md

echo "$commits" >> commits.md

echo "commits.md generated"
