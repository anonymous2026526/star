cd ../
for A in 0x401C5A4 0x4017CD5 0x4015AB5 0x4018795; do
  F=$(printf "0x%x" $((A - 0x4000000)))
  echo "=== A=$A F=$F ==="
  objdump -Cd --demangle ./target/release/analysis \
    --start-address=$((F-512)) \
    --stop-address=$((F+512))
done
