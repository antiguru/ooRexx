/* extracted from CONCATENATION::test_1 */
::routine main public
   a="abcdefg"'00'x
   b="abcdefgh"
   c="abcdefg "
   d=" abcdefg"
   e='00'x"abcdefg"
   f=" abcdefg "
   g='00'x"abcdefg"'00'x
   self~assertSame((a==a) (b==a) (c==a) (d==a) (e==a) (f==a) (g==a), '1 0 0 0 0 0 0')
   self~assertSame((a==b) (b==b) (c==b) (d==b) (e==b) (f==b) (g==b), '0 1 0 0 0 0 0')
   self~assertSame((a==c) (b==c) (c==c) (d==c) (e==c) (f==c) (g==c), '0 0 1 0 0 0 0')
   self~assertSame((a==d) (b==d) (c==d) (d==d) (e==d) (f==d) (g==d), '0 0 0 1 0 0 0')
   self~assertSame((a==e) (b==e) (c==e) (d==e) (e==e) (f==e) (g==e), '0 0 0 0 1 0 0')
   self~assertSame((a==f) (b==f) (c==f) (d==f) (e==f) (f==f) (g==f), '0 0 0 0 0 1 0')
   self~assertSame((a==g) (b==g) (c==g) (d==g) (e==g) (f==g) (g==g), '0 0 0 0 0 0 1')
   self~assertSame((a=a) (b=a) (c=a) (d=a) (e=a) (f=a) (g=a), '1 0 0 0 0 0 0')
   self~assertSame((a=b) (b=b) (c=b) (d=b) (e=b) (f=b) (g=b), '0 1 0 0 0 0 0')
   self~assertSame((a=c) (b=c) (c=c) (d=c) (e=c) (f=c) (g=c), '0 0 1 1 0 1 0')
   self~assertSame((a=d) (b=d) (c=d) (d=d) (e=d) (f=d) (g=d), '0 0 1 1 0 1 0')
   self~assertSame((a=e) (b=e) (c=e) (d=e) (e=e) (f=e) (g=e), '0 0 0 0 1 0 0')
   self~assertSame((a=f) (b=f) (c=f) (d=f) (e=f) (f=f) (g=f), '0 0 1 1 0 1 0')
   self~assertSame((a=g) (b=g) (c=g) (d=g) (e=g) (f=g) (g=g), '0 0 0 0 0 0 1')
   self~assertSame((a\==a) (b\==a) (c\==a) (d\==a) (e\==a) (f\==a) (g\==a), '0 1 1 1 1 1 1')
   self~assertSame((a\==b) (b\==b) (c\==b) (d\==b) (e\==b) (f\==b) (g\==b), '1 0 1 1 1 1 1')
   self~assertSame((a\==c) (b\==c) (c\==c) (d\==c) (e\==c) (f\==c) (g\==c), '1 1 0 1 1 1 1')
   self~assertSame((a\==d) (b\==d) (c\==d) (d\==d) (e\==d) (f\==d) (g\==d), '1 1 1 0 1 1 1')
   self~assertSame((a\==e) (b\==e) (c\==e) (d\==e) (e\==e) (f\==e) (g\==e), '1 1 1 1 0 1 1')
   self~assertSame((a\==f) (b\==f) (c\==f) (d\==f) (e\==f) (f\==f) (g\==f), '1 1 1 1 1 0 1')
   self~assertSame((a\==g) (b\==g) (c\==g) (d\==g) (e\==g) (f\==g) (g\==g), '1 1 1 1 1 1 0')
   self~assertSame((a\==a) (b\==a) (c\==a) (d\==a) (e\==a) (f\==a) (g\==a), '0 1 1 1 1 1 1')
   self~assertSame((a\==b) (b\==b) (c\==b) (d\==b) (e\==b) (f\==b) (g\==b), '1 0 1 1 1 1 1')
   self~assertSame((a\==c) (b\==c) (c\==c) (d\==c) (e\==c) (f\==c) (g\==c), '1 1 0 1 1 1 1')
   self~assertSame((a\==d) (b\==d) (c\==d) (d\==d) (e\==d) (f\==d) (g\==d), '1 1 1 0 1 1 1')
   self~assertSame((a\==e) (b\==e) (c\==e) (d\==e) (e\==e) (f\==e) (g\==e), '1 1 1 1 0 1 1')
   self~assertSame((a\==f) (b\==f) (c\==f) (d\==f) (e\==f) (f\==f) (g\==f), '1 1 1 1 1 0 1')
   self~assertSame((a\==g) (b\==g) (c\==g) (d\==g) (e\==g) (f\==g) (g\==g), '1 1 1 1 1 1 0')
   self~assertSame((a\==a) (b\==a) (c\==a) (d\==a) (e\==a) (f\==a) (g\==a), '0 1 1 1 1 1 1')
   self~assertSame((a\==b) (b\==b) (c\==b) (d\==b) (e\==b) (f\==b) (g\==b), '1 0 1 1 1 1 1')
   self~assertSame((a\==c) (b\==c) (c\==c) (d\==c) (e\==c) (f\==c) (g\==c), '1 1 0 1 1 1 1')
   self~assertSame((a\==d) (b\==d) (c\==d) (d\==d) (e\==d) (f\==d) (g\==d), '1 1 1 0 1 1 1')
   self~assertSame((a\==e) (b\==e) (c\==e) (d\==e) (e\==e) (f\==e) (g\==e), '1 1 1 1 0 1 1')
   self~assertSame((a\==f) (b\==f) (c\==f) (d\==f) (e\==f) (f\==f) (g\==f), '1 1 1 1 1 0 1')
   self~assertSame((a\==g) (b\==g) (c\==g) (d\==g) (e\==g) (f\==g) (g\==g), '1 1 1 1 1 1 0')
   self~assertSame((a\=a) (b\=a) (c\=a) (d\=a) (e\=a) (f\=a) (g\=a), '0 1 1 1 1 1 1')
   self~assertSame((a\=b) (b\=b) (c\=b) (d\=b) (e\=b) (f\=b) (g\=b), '1 0 1 1 1 1 1')
   self~assertSame((a\=c) (b\=c) (c\=c) (d\=c) (e\=c) (f\=c) (g\=c), '1 1 0 0 1 0 1')
   self~assertSame((a\=d) (b\=d) (c\=d) (d\=d) (e\=d) (f\=d) (g\=d), '1 1 0 0 1 0 1')
   self~assertSame((a\=e) (b\=e) (c\=e) (d\=e) (e\=e) (f\=e) (g\=e), '1 1 1 1 0 1 1')
   self~assertSame((a\=f) (b\=f) (c\=f) (d\=f) (e\=f) (f\=f) (g\=f), '1 1 0 0 1 0 1')
   self~assertSame((a\=g) (b\=g) (c\=g) (d\=g) (e\=g) (f\=g) (g\=g), '1 1 1 1 1 1 0')
   self~assertSame((a\=a) (b\=a) (c\=a) (d\=a) (e\=a) (f\=a) (g\=a), '0 1 1 1 1 1 1')
   self~assertSame((a\=b) (b\=b) (c\=b) (d\=b) (e\=b) (f\=b) (g\=b), '1 0 1 1 1 1 1')
   self~assertSame((a\=c) (b\=c) (c\=c) (d\=c) (e\=c) (f\=c) (g\=c), '1 1 0 0 1 0 1')
   self~assertSame((a\=d) (b\=d) (c\=d) (d\=d) (e\=d) (f\=d) (g\=d), '1 1 0 0 1 0 1')
   self~assertSame((a\=e) (b\=e) (c\=e) (d\=e) (e\=e) (f\=e) (g\=e), '1 1 1 1 0 1 1')
   self~assertSame((a\=f) (b\=f) (c\=f) (d\=f) (e\=f) (f\=f) (g\=f), '1 1 0 0 1 0 1')
   self~assertSame((a\=g) (b\=g) (c\=g) (d\=g) (e\=g) (f\=g) (g\=g), '1 1 1 1 1 1 0')
   self~assertSame((a\=a) (b\=a) (c\=a) (d\=a) (e\=a) (f\=a) (g\=a), '0 1 1 1 1 1 1')
   self~assertSame((a\=b) (b\=b) (c\=b) (d\=b) (e\=b) (f\=b) (g\=b), '1 0 1 1 1 1 1')
   self~assertSame((a\=c) (b\=c) (c\=c) (d\=c) (e\=c) (f\=c) (g\=c), '1 1 0 0 1 0 1')
   self~assertSame((a\=d) (b\=d) (c\=d) (d\=d) (e\=d) (f\=d) (g\=d), '1 1 0 0 1 0 1')
   self~assertSame((a\=e) (b\=e) (c\=e) (d\=e) (e\=e) (f\=e) (g\=e), '1 1 1 1 0 1 1')
   self~assertSame((a\=f) (b\=f) (c\=f) (d\=f) (e\=f) (f\=f) (g\=f), '1 1 0 0 1 0 1')
   self~assertSame((a\=g) (b\=g) (c\=g) (d\=g) (e\=g) (f\=g) (g\=g), '1 1 1 1 1 1 0')
   self~assertSame((a>>a) (b>>a) (c>>a) (d>>a) (e>>a) (f>>a) (g>>a), '0 1 1 0 0 0 0')
   self~assertSame((a>>b) (b>>b) (c>>b) (d>>b) (e>>b) (f>>b) (g>>b), '0 0 0 0 0 0 0')
   self~assertSame((a>>c) (b>>c) (c>>c) (d>>c) (e>>c) (f>>c) (g>>c), '0 1 0 0 0 0 0')
   self~assertSame((a>>d) (b>>d) (c>>d) (d>>d) (e>>d) (f>>d) (g>>d), '1 1 1 0 0 1 0')
   self~assertSame((a>>e) (b>>e) (c>>e) (d>>e) (e>>e) (f>>e) (g>>e), '1 1 1 1 0 1 1')
   self~assertSame((a>>f) (b>>f) (c>>f) (d>>f) (e>>f) (f>>f) (g>>f), '1 1 1 0 0 0 0')
   self~assertSame((a>>g) (b>>g) (c>>g) (d>>g) (e>>g) (f>>g) (g>>g), '1 1 1 1 0 1 0')
   self~assertSame((a>a) (b>a) (c>a) (d>a) (e>a) (f>a) (g>a), '0 1 1 1 0 1 0')
   self~assertSame((a>b) (b>b) (c>b) (d>b) (e>b) (f>b) (g>b), '0 0 0 0 0 0 0')
   self~assertSame((a>c) (b>c) (c>c) (d>c) (e>c) (f>c) (g>c), '0 1 0 0 0 0 0')
   self~assertSame((a>d) (b>d) (c>d) (d>d) (e>d) (f>d) (g>d), '0 1 0 0 0 0 0')
   self~assertSame((a>e) (b>e) (c>e) (d>e) (e>e) (f>e) (g>e), '1 1 1 1 0 1 0')
   self~assertSame((a>f) (b>f) (c>f) (d>f) (e>f) (f>f) (g>f), '0 1 0 0 0 0 0')
   self~assertSame((a>g) (b>g) (c>g) (d>g) (e>g) (f>g) (g>g), '1 1 1 1 1 1 0')
   self~assertSame((a<<a) (b<<a) (c<<a) (d<<a) (e<<a) (f<<a) (g<<a), '0 0 0 1 1 1 1')
   self~assertSame((a<<b) (b<<b) (c<<b) (d<<b) (e<<b) (f<<b) (g<<b), '1 0 1 1 1 1 1')
   self~assertSame((a<<c) (b<<c) (c<<c) (d<<c) (e<<c) (f<<c) (g<<c), '1 0 0 1 1 1 1')
   self~assertSame((a<<d) (b<<d) (c<<d) (d<<d) (e<<d) (f<<d) (g<<d), '0 0 0 0 1 0 1')
   self~assertSame((a<<e) (b<<e) (c<<e) (d<<e) (e<<e) (f<<e) (g<<e), '0 0 0 0 0 0 0')
   self~assertSame((a<<f) (b<<f) (c<<f) (d<<f) (e<<f) (f<<f) (g<<f), '0 0 0 1 1 0 1')
   self~assertSame((a<<g) (b<<g) (c<<g) (d<<g) (e<<g) (f<<g) (g<<g), '0 0 0 0 1 0 0')
   self~assertSame((a<a) (b<a) (c<a) (d<a) (e<a) (f<a) (g<a), '0 0 0 0 1 0 1')
   self~assertSame((a<b) (b<b) (c<b) (d<b) (e<b) (f<b) (g<b), '1 0 1 1 1 1 1')
   self~assertSame((a<c) (b<c) (c<c) (d<c) (e<c) (f<c) (g<c), '1 0 0 0 1 0 1')
   self~assertSame((a<d) (b<d) (c<d) (d<d) (e<d) (f<d) (g<d), '1 0 0 0 1 0 1')
   self~assertSame((a<e) (b<e) (c<e) (d<e) (e<e) (f<e) (g<e), '0 0 0 0 0 0 1')
   self~assertSame((a<f) (b<f) (c<f) (d<f) (e<f) (f<f) (g<f), '1 0 0 0 1 0 1')
   self~assertSame((a<g) (b<g) (c<g) (d<g) (e<g) (f<g) (g<g), '0 0 0 0 0 0 0')
   self~assertSame((a><a) (b><a) (c><a) (d><a) (e><a) (f><a) (g><a), '0 1 1 1 1 1 1')
   self~assertSame((a><b) (b><b) (c><b) (d><b) (e><b) (f><b) (g><b), '1 0 1 1 1 1 1')
   self~assertSame((a><c) (b><c) (c><c) (d><c) (e><c) (f><c) (g><c), '1 1 0 0 1 0 1')
   self~assertSame((a><d) (b><d) (c><d) (d><d) (e><d) (f><d) (g><d), '1 1 0 0 1 0 1')
   self~assertSame((a><e) (b><e) (c><e) (d><e) (e><e) (f><e) (g><e), '1 1 1 1 0 1 1')
   self~assertSame((a><f) (b><f) (c><f) (d><f) (e><f) (f><f) (g><f), '1 1 0 0 1 0 1')
   self~assertSame((a><g) (b><g) (c><g) (d><g) (e><g) (f><g) (g><g), '1 1 1 1 1 1 0')
   self~assertSame((a<>a) (b<>a) (c<>a) (d<>a) (e<>a) (f<>a) (g<>a), '0 1 1 1 1 1 1')
   self~assertSame((a<>b) (b<>b) (c<>b) (d<>b) (e<>b) (f<>b) (g<>b), '1 0 1 1 1 1 1')
   self~assertSame((a<>c) (b<>c) (c<>c) (d<>c) (e<>c) (f<>c) (g<>c), '1 1 0 0 1 0 1')
   self~assertSame((a<>d) (b<>d) (c<>d) (d<>d) (e<>d) (f<>d) (g<>d), '1 1 0 0 1 0 1')
   self~assertSame((a<>e) (b<>e) (c<>e) (d<>e) (e<>e) (f<>e) (g<>e), '1 1 1 1 0 1 1')
   self~assertSame((a<>f) (b<>f) (c<>f) (d<>f) (e<>f) (f<>f) (g<>f), '1 1 0 0 1 0 1')
   self~assertSame((a<>g) (b<>g) (c<>g) (d<>g) (e<>g) (f<>g) (g<>g), '1 1 1 1 1 1 0')
   self~assertSame((a>>=a) (b>>=a) (c>>=a) (d>>=a) (e>>=a) (f>>=a) (g>>=a), '1 1 1 0 0 0 0')
   self~assertSame((a>>=b) (b>>=b) (c>>=b) (d>>=b) (e>>=b) (f>>=b) (g>>=b), '0 1 0 0 0 0 0')
   self~assertSame((a>>=c) (b>>=c) (c>>=c) (d>>=c) (e>>=c) (f>>=c) (g>>=c), '0 1 1 0 0 0 0')
   self~assertSame((a>>=d) (b>>=d) (c>>=d) (d>>=d) (e>>=d) (f>>=d) (g>>=d), '1 1 1 1 0 1 0')
   self~assertSame((a>>=e) (b>>=e) (c>>=e) (d>>=e) (e>>=e) (f>>=e) (g>>=e), '1 1 1 1 1 1 1')
   self~assertSame((a>>=f) (b>>=f) (c>>=f) (d>>=f) (e>>=f) (f>>=f) (g>>=f), '1 1 1 0 0 1 0')
   self~assertSame((a>>=g) (b>>=g) (c>>=g) (d>>=g) (e>>=g) (f>>=g) (g>>=g), '1 1 1 1 0 1 1')
   self~assertSame((a>=a) (b>=a) (c>=a) (d>=a) (e>=a) (f>=a) (g>=a), '1 1 1 1 0 1 0')
   self~assertSame((a>=b) (b>=b) (c>=b) (d>=b) (e>=b) (f>=b) (g>=b), '0 1 0 0 0 0 0')
   self~assertSame((a>=c) (b>=c) (c>=c) (d>=c) (e>=c) (f>=c) (g>=c), '0 1 1 1 0 1 0')
   self~assertSame((a>=d) (b>=d) (c>=d) (d>=d) (e>=d) (f>=d) (g>=d), '0 1 1 1 0 1 0')
   self~assertSame((a>=e) (b>=e) (c>=e) (d>=e) (e>=e) (f>=e) (g>=e), '1 1 1 1 1 1 0')
   self~assertSame((a>=f) (b>=f) (c>=f) (d>=f) (e>=f) (f>=f) (g>=f), '0 1 1 1 0 1 0')
   self~assertSame((a>=g) (b>=g) (c>=g) (d>=g) (e>=g) (f>=g) (g>=g), '1 1 1 1 1 1 1')
   self~assertSame((a<<=a) (b<<=a) (c<<=a) (d<<=a) (e<<=a) (f<<=a) (g<<=a), '1 0 0 1 1 1 1')
   self~assertSame((a<<=b) (b<<=b) (c<<=b) (d<<=b) (e<<=b) (f<<=b) (g<<=b), '1 1 1 1 1 1 1')
   self~assertSame((a<<=c) (b<<=c) (c<<=c) (d<<=c) (e<<=c) (f<<=c) (g<<=c), '1 0 1 1 1 1 1')
   self~assertSame((a<<=d) (b<<=d) (c<<=d) (d<<=d) (e<<=d) (f<<=d) (g<<=d), '0 0 0 1 1 0 1')
   self~assertSame((a<<=e) (b<<=e) (c<<=e) (d<<=e) (e<<=e) (f<<=e) (g<<=e), '0 0 0 0 1 0 0')
   self~assertSame((a<<=f) (b<<=f) (c<<=f) (d<<=f) (e<<=f) (f<<=f) (g<<=f), '0 0 0 1 1 1 1')
   self~assertSame((a<<=g) (b<<=g) (c<<=g) (d<<=g) (e<<=g) (f<<=g) (g<<=g), '0 0 0 0 1 0 1')
   self~assertSame((a<=a) (b<=a) (c<=a) (d<=a) (e<=a) (f<=a) (g<=a), '1 0 0 0 1 0 1')
   self~assertSame((a<=b) (b<=b) (c<=b) (d<=b) (e<=b) (f<=b) (g<=b), '1 1 1 1 1 1 1')
   self~assertSame((a<=c) (b<=c) (c<=c) (d<=c) (e<=c) (f<=c) (g<=c), '1 0 1 1 1 1 1')
   self~assertSame((a<=d) (b<=d) (c<=d) (d<=d) (e<=d) (f<=d) (g<=d), '1 0 1 1 1 1 1')
   self~assertSame((a<=e) (b<=e) (c<=e) (d<=e) (e<=e) (f<=e) (g<=e), '0 0 0 0 1 0 1')
   self~assertSame((a<=f) (b<=f) (c<=f) (d<=f) (e<=f) (f<=f) (g<=f), '1 0 1 1 1 1 1')
   self~assertSame((a<=g) (b<=g) (c<=g) (d<=g) (e<=g) (f<=g) (g<=g), '0 0 0 0 0 0 1')
   self~assertSame((a\>>a) (b\>>a) (c\>>a) (d\>>a) (e\>>a) (f\>>a) (g\>>a), '1 0 0 1 1 1 1')
   self~assertSame((a\>>b) (b\>>b) (c\>>b) (d\>>b) (e\>>b) (f\>>b) (g\>>b), '1 1 1 1 1 1 1')
   self~assertSame((a\>>c) (b\>>c) (c\>>c) (d\>>c) (e\>>c) (f\>>c) (g\>>c), '1 0 1 1 1 1 1')
   self~assertSame((a\>>d) (b\>>d) (c\>>d) (d\>>d) (e\>>d) (f\>>d) (g\>>d), '0 0 0 1 1 0 1')
   self~assertSame((a\>>e) (b\>>e) (c\>>e) (d\>>e) (e\>>e) (f\>>e) (g\>>e), '0 0 0 0 1 0 0')
   self~assertSame((a\>>f) (b\>>f) (c\>>f) (d\>>f) (e\>>f) (f\>>f) (g\>>f), '0 0 0 1 1 1 1')
   self~assertSame((a\>>g) (b\>>g) (c\>>g) (d\>>g) (e\>>g) (f\>>g) (g\>>g), '0 0 0 0 1 0 1')
   self~assertSame((a\>a) (b\>a) (c\>a) (d\>a) (e\>a) (f\>a) (g\>a), '1 0 0 0 1 0 1')
   self~assertSame((a\>b) (b\>b) (c\>b) (d\>b) (e\>b) (f\>b) (g\>b), '1 1 1 1 1 1 1')
   self~assertSame((a\>c) (b\>c) (c\>c) (d\>c) (e\>c) (f\>c) (g\>c), '1 0 1 1 1 1 1')
   self~assertSame((a\>d) (b\>d) (c\>d) (d\>d) (e\>d) (f\>d) (g\>d), '1 0 1 1 1 1 1')
   self~assertSame((a\>e) (b\>e) (c\>e) (d\>e) (e\>e) (f\>e) (g\>e), '0 0 0 0 1 0 1')
   self~assertSame((a\>f) (b\>f) (c\>f) (d\>f) (e\>f) (f\>f) (g\>f), '1 0 1 1 1 1 1')
   self~assertSame((a\>g) (b\>g) (c\>g) (d\>g) (e\>g) (f\>g) (g\>g), '0 0 0 0 0 0 1')
   self~assertSame((a\<<a) (b\<<a) (c\<<a) (d\<<a) (e\<<a) (f\<<a) (g\<<a), '1 1 1 0 0 0 0')
   self~assertSame((a\<<b) (b\<<b) (c\<<b) (d\<<b) (e\<<b) (f\<<b) (g\<<b), '0 1 0 0 0 0 0')
   self~assertSame((a\<<c) (b\<<c) (c\<<c) (d\<<c) (e\<<c) (f\<<c) (g\<<c), '0 1 1 0 0 0 0')
   self~assertSame((a\<<d) (b\<<d) (c\<<d) (d\<<d) (e\<<d) (f\<<d) (g\<<d), '1 1 1 1 0 1 0')
   self~assertSame((a\<<e) (b\<<e) (c\<<e) (d\<<e) (e\<<e) (f\<<e) (g\<<e), '1 1 1 1 1 1 1')
   self~assertSame((a\<<f) (b\<<f) (c\<<f) (d\<<f) (e\<<f) (f\<<f) (g\<<f), '1 1 1 0 0 1 0')
   self~assertSame((a\<<g) (b\<<g) (c\<<g) (d\<<g) (e\<<g) (f\<<g) (g\<<g), '1 1 1 1 0 1 1')
   self~assertSame((a\<a) (b\<a) (c\<a) (d\<a) (e\<a) (f\<a) (g\<a), '1 1 1 1 0 1 0')
   self~assertSame((a\<b) (b\<b) (c\<b) (d\<b) (e\<b) (f\<b) (g\<b), '0 1 0 0 0 0 0')
   self~assertSame((a\<c) (b\<c) (c\<c) (d\<c) (e\<c) (f\<c) (g\<c), '0 1 1 1 0 1 0')
   self~assertSame((a\<d) (b\<d) (c\<d) (d\<d) (e\<d) (f\<d) (g\<d), '0 1 1 1 0 1 0')
   self~assertSame((a\<e) (b\<e) (c\<e) (d\<e) (e\<e) (f\<e) (g\<e), '1 1 1 1 1 1 0')
   self~assertSame((a\<f) (b\<f) (c\<f) (d\<f) (e\<f) (f\<f) (g\<f), '0 1 1 1 0 1 0')
   self~assertSame((a\<g) (b\<g) (c\<g) (d\<g) (e\<g) (f\<g) (g\<g), '1 1 1 1 1 1 1')
   self~assertSame((a\>>a) (b\>>a) (c\>>a) (d\>>a) (e\>>a) (f\>>a) (g\>>a), '1 0 0 1 1 1 1')
   self~assertSame((a\>>b) (b\>>b) (c\>>b) (d\>>b) (e\>>b) (f\>>b) (g\>>b), '1 1 1 1 1 1 1')
   self~assertSame((a\>>c) (b\>>c) (c\>>c) (d\>>c) (e\>>c) (f\>>c) (g\>>c), '1 0 1 1 1 1 1')
   self~assertSame((a\>>d) (b\>>d) (c\>>d) (d\>>d) (e\>>d) (f\>>d) (g\>>d), '0 0 0 1 1 0 1')
   self~assertSame((a\>>e) (b\>>e) (c\>>e) (d\>>e) (e\>>e) (f\>>e) (g\>>e), '0 0 0 0 1 0 0')
   self~assertSame((a\>>f) (b\>>f) (c\>>f) (d\>>f) (e\>>f) (f\>>f) (g\>>f), '0 0 0 1 1 1 1')
   self~assertSame((a\>>g) (b\>>g) (c\>>g) (d\>>g) (e\>>g) (f\>>g) (g\>>g), '0 0 0 0 1 0 1')
   self~assertSame((a\>a) (b\>a) (c\>a) (d\>a) (e\>a) (f\>a) (g\>a), '1 0 0 0 1 0 1')
   self~assertSame((a\>b) (b\>b) (c\>b) (d\>b) (e\>b) (f\>b) (g\>b), '1 1 1 1 1 1 1')
   self~assertSame((a\>c) (b\>c) (c\>c) (d\>c) (e\>c) (f\>c) (g\>c), '1 0 1 1 1 1 1')
   self~assertSame((a\>d) (b\>d) (c\>d) (d\>d) (e\>d) (f\>d) (g\>d), '1 0 1 1 1 1 1')
   self~assertSame((a\>e) (b\>e) (c\>e) (d\>e) (e\>e) (f\>e) (g\>e), '0 0 0 0 1 0 1')
   self~assertSame((a\>f) (b\>f) (c\>f) (d\>f) (e\>f) (f\>f) (g\>f), '1 0 1 1 1 1 1')
   self~assertSame((a\>g) (b\>g) (c\>g) (d\>g) (e\>g) (f\>g) (g\>g), '0 0 0 0 0 0 1')
   self~assertSame((a\<<a) (b\<<a) (c\<<a) (d\<<a) (e\<<a) (f\<<a) (g\<<a), '1 1 1 0 0 0 0')
   self~assertSame((a\<<b) (b\<<b) (c\<<b) (d\<<b) (e\<<b) (f\<<b) (g\<<b), '0 1 0 0 0 0 0')
   self~assertSame((a\<<c) (b\<<c) (c\<<c) (d\<<c) (e\<<c) (f\<<c) (g\<<c), '0 1 1 0 0 0 0')
   self~assertSame((a\<<d) (b\<<d) (c\<<d) (d\<<d) (e\<<d) (f\<<d) (g\<<d), '1 1 1 1 0 1 0')
   self~assertSame((a\<<e) (b\<<e) (c\<<e) (d\<<e) (e\<<e) (f\<<e) (g\<<e), '1 1 1 1 1 1 1')
   self~assertSame((a\<<f) (b\<<f) (c\<<f) (d\<<f) (e\<<f) (f\<<f) (g\<<f), '1 1 1 0 0 1 0')
   self~assertSame((a\<<g) (b\<<g) (c\<<g) (d\<<g) (e\<<g) (f\<<g) (g\<<g), '1 1 1 1 0 1 1')
   self~assertSame((a\<a) (b\<a) (c\<a) (d\<a) (e\<a) (f\<a) (g\<a), '1 1 1 1 0 1 0')
   self~assertSame((a\<b) (b\<b) (c\<b) (d\<b) (e\<b) (f\<b) (g\<b), '0 1 0 0 0 0 0')
   self~assertSame((a\<c) (b\<c) (c\<c) (d\<c) (e\<c) (f\<c) (g\<c), '0 1 1 1 0 1 0')
   self~assertSame((a\<d) (b\<d) (c\<d) (d\<d) (e\<d) (f\<d) (g\<d), '0 1 1 1 0 1 0')
   self~assertSame((a\<e) (b\<e) (c\<e) (d\<e) (e\<e) (f\<e) (g\<e), '1 1 1 1 1 1 0')
   self~assertSame((a\<f) (b\<f) (c\<f) (d\<f) (e\<f) (f\<f) (g\<f), '0 1 1 1 0 1 0')
   self~assertSame((a\<g) (b\<g) (c\<g) (d\<g) (e\<g) (f\<g) (g\<g), '1 1 1 1 1 1 1')

::class shim public
::method assertEquals
  use arg expected, actual
  if expected \== actual then do
    say "FAIL expected["expected"] actual["actual"]"
    exit 1
  end
::method assertNotEquals
  use arg expected, actual
  if expected == actual then do
    say "FAIL not-expected["expected"] actual["actual"]"
    exit 1
  end
::method assertTrue
  use arg condition
  if \condition then do
    say "FAIL expected true actual["condition"]"
    exit 1
  end
::method assertFalse
  use arg condition
  if condition then do
    say "FAIL expected false actual["condition"]"
    exit 1
  end
::method assertNull
  use arg actual
  if actual \== .nil then do
    say "FAIL expected nil actual["actual"]"
    exit 1
  end
::method assertNotNull
  use arg actual
  if actual == .nil then do
    say "FAIL expected non-nil actual nil"
    exit 1
  end
::method assertSame
  use arg expected, actual
  if \(expected == actual) then do
    say "FAIL expected["expected"] actual["actual"]"
    exit 1
  end
::method assertNotSame
  use arg expected, actual
  if expected == actual then do
    say "FAIL not-expected["expected"] actual["actual"]"
    exit 1
  end
::method expectSyntax
  use arg code
  nop
::method assertListEquals
  use arg expected, actual
  if expected \== actual then do
    say "FAIL expected["expected"] actual["actual"]"
    exit 1
  end
::method assertArrayEquals
  use arg expected, actual
  if expected \== actual then do
    say "FAIL expected["expected"] actual["actual"]"
    exit 1
  end
