import sympy as sp

u = sp.Symbol("u")
ti, ti1, ti2, ti3, ti4, ti5, ti6 = sp.symbols("ti ti1 ti2 ti3 ti4 ti5 ti6")

di, di1, di2, di3 = sp.symbols("di di1 di2 di3")

y = sp.Symbol("y")

def blend(A, B, tir, tin):
    return (1 - (u - tir)/(tin - tir)) * A + ((u - tir)/(tin - tir)) * B




# De Boor (degree 3): three levels of blends
d0_1 = blend(di,  di1, ti1,  ti4)
d1_1 = blend(di1, di2, ti2, ti5)
d2_1 = blend(di2, di3, ti3, ti6)

d0_2 = blend(d0_1, d1_1, ti2,  ti4)
d1_2 = blend(d1_1, d2_1, ti3, ti5)

d0_3 = blend(d0_2, d1_2, ti3, ti4)   # final cubic

solutions = sp.solve(sp.Eq(d0_3, y), u)
print(".....................")

for i, sol in enumerate(solutions):
    print(f"Solution {i}:")
    print(sp.rust_code(sol))

