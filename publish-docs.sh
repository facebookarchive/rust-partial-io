#!/bin/bash
# Copyright (c) Facebook, Inc. and its affiliates.
#
# This source code is licensed under the MIT license found in the
# LICENSE file in the root directory of this source tree.

# stop running if any of these steps fail
set -e
pushd ..

if test ! -d partial-io-gh-pages ; then
  git clone -b gh-pages git@github.com:facebookincubator/rust-partial-io.git partial-io-gh-pages
fi
cd partial-io-gh-pages

git checkout --
git clean -dfx
git fetch
git rebase origin/gh-pages
popd
cargo doc --lib --features 'tokio quickcheck' --no-deps
rsync -aH --delete target/doc/ --exclude .git ../partial-io-gh-pages
# cargo doc doesn't write an index file by default
echo '<meta http-equiv=refresh content=0;url=partial_io/index.html>' > ../partial-io-gh-pages/index.html
pushd ../partial-io-gh-pages

git add --all
git commit -m "update website"
git push origin gh-pages
