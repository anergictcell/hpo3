Info about likely API changes
=============================

This section contains info about likely API changes that will happen in the
future and how you can make sure to write your code forward compatible

List will become Iterator
-------------------------

Some functions return lists of items, e.g. ``List[HPOTerm]``, ``List[Gene]```,
but will change to return an Iterator instead.
In most cases, this should not affect how you use the library and you will not
notice a difference. It does come with a few restrictions, though:

1. An iterator is not subsettable: ``x = iterator[0]`` will not work
2. An iterator does not have a length: ``len(iterator)`` will not work

If a method indicates that the return type will change to an Iterator
you should not subset the result, even if it is a list. Subsetting the
results of those functions is not a good idea anyway, because in most cases
the order of return items is not guaranteed and might change randomly.

If you ever need to subset such a return type, convert it to a list:

.. code-block:: python

    def terms() -> Iterator[HPOTerm]:
        ...

    # Don't
    x = terms()[0]

    # Use instead:
    x = list(terms())[0]
