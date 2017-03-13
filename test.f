
      OPEN(UNIT=8, FILE='test.fdat',
     1form="unformatted",access="sequential") 
      WRITE(8) 2, 3
      WRITE(8) 5
      PRINT *, "Hello World!"
      END