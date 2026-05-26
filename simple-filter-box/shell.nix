{ pkgs ? import <nixpkgs> {} }:
let
  plotsDir = "./target/criterion/plots";
in
pkgs.mkShell {
  packages = [
    pkgs.cargo
    pkgs.rustc
    pkgs.gnuplot
    pkgs.python3
  ];
  shellHook = ''
    export NIX_ENFORCE_PURITY=0

    CRITERION_DEBUG=1 cargo bench --bench filter_bench -- --plotting-backend gnuplot
    criterionDir=./target/criterion
    testDir="$criterionDir/test"
    insertDir="$criterionDir/insert"
    setYRangeCmd='/^set grid ytics$/a\set yrange [0:*]'
    mkdir -p ${plotsDir}
    cp "$testDir/bloom_miss/report/lines.gnuplot" ${plotsDir}/test_bloom_miss.gnuplot
    cp "$testDir/redb_miss/report/lines.gnuplot" ${plotsDir}/test_redb_miss.gnuplot
    cp "$insertDir/bloom/report/lines.gnuplot" ${plotsDir}/insert_bloom.gnuplot
    cp "$insertDir/redb/report/lines.gnuplot" ${plotsDir}/insert_redb.gnuplot

    cd ${plotsDir}

    sed -i "$setYRangeCmd" ./test_bloom_miss.gnuplot
    gnuplot ./test_bloom_miss.gnuplot
    mv output.plot ./test_bloom_miss.svg

    sed -i "$setYRangeCmd" ./test_redb_miss.gnuplot
    gnuplot ./test_redb_miss.gnuplot
    mv output.plot ./test_redb_miss.svg

    sed -i "$setYRangeCmd" ./insert_bloom.gnuplot
    gnuplot ./insert_bloom.gnuplot
    mv output.plot ./insert_bloom.svg

    sed -i "$setYRangeCmd" ./insert_redb.gnuplot
    gnuplot ./insert_redb.gnuplot
    mv output.plot ./insert_redb.svg
  '';
}
