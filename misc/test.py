import sympy as sp

u = sp.Symbol("u")
ti, ti1, ti2, ti3, ti4, ti5, ti6 = sp.symbols("ti ti1 ti2 ti3 ti4 ti5 ti6")
di, di1, di2, di3 = sp.symbols("di di1 di2 di3")

p = sp.symbols("p")

# def blend(A, B, a, b):
#     return (1 - (u - a)/(b - a)) * A + ((u - a)/(b - a)) * B

def blend(A, B, tir, tin):
    return (1 - (u - tir)/(tin - tir)) * A + ((u - tir)/(tin - tir)) * B




# De Boor (degree 3): three levels of blends
d0_1 = blend(di,  di1, ti1,  ti4)
d1_1 = blend(di1, di2, ti2, ti5)
d2_1 = blend(di2, di3, ti3, ti6)

d0_2 = blend(d0_1, d1_1, ti2,  ti4)
d1_2 = blend(d1_1, d2_1, ti3, ti5)

d0_3 = blend(d0_2, d1_2, ti3, ti4)   # final cubic

# expanded = sp.expand(d0_3)




rust = sp.rust_code(sp.simplify(d0_3))
print(rust)

print("_________distance______")

distance = d0_3 - p ?????


print("_________derivative_distance______")
derivative = sp.diff(d0_3, u)
derivative_simplified = sp.simplify(derivative)
print(sp.rust_code(derivative_simplified))

