* StatCraft Canonical Batch Analysis Script.
* Demonstrates multiple statistical procedures and data transformations.

DESCRIPTIVES VARIABLES=salary age
  /STATISTICS=MEAN STDDEV MIN MAX.

FREQUENCIES VARIABLES=gender.

SORT CASES BY salary (D) age (A).

CROSSTABS
  /TABLES=jobcat BY gender
  /STATISTICS=CHISQ.
