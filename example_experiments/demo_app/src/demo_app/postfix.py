#! /usr/bin/python3


class Unit:
    def __init__(self, unit, value):
        self.unit = unit
        self.value = value

    def __str__(self):
        return f"{self.value} {self.unit}"

    # allow adding units together
    def __add__(self, other):
        if self.unit != other.unit:
            raise ValueError("Units must be the same")
        return Unit(self.unit, self.value + other.value)

    def __sub__(self, other):
        if self.unit != other.unit:
            raise ValueError("Units must be the same")
        return Unit(self.unit, self.value - other.value)

    def __mul__(self, other):
        return Unit(self.unit, self.value * other)

    def __rmul__(self, other):
        return Unit(self.unit, self.value * other)


class UnitFactory:
    def __init__(self, unit):
        self.unit = unit

    def __rmul__(self, other):
        return Unit(self.unit, other)

    def __str__(self):
        return self.unit


px = UnitFactory("px")
sw = UnitFactory("Screen Width")
sh = UnitFactory("Screen Height")
cm = UnitFactory("cm")
mm = UnitFactory("mm")

length = 2 * (3 * cm) - 4 * cm

print(length)
