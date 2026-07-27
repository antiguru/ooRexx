/* Decimal-arithmetic dimension: NUMERIC DIGITS switched mid-loop, so both
   settings are exercised every iteration rather than only at startup. */
n = 500000
total = 0
do i = 1 to n
    numeric digits 9
    a = i / 3
    b = a * a - 1
    numeric digits 20
    c = i / 7
    d = c ** 2 // 5
    total = total + b + d
end
say total
