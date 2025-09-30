# 1. 编译为 .air
xcrun -sdk macosx metal -c bit_reverse.metal -o bit_reverse.air

# 2. 链接为 .metallib
xcrun -sdk macosx metallib bit_reverse.air -o bit_reverse.metallib