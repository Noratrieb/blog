main: multia.c libmultib.a libmultic.a libmultib.so libmultic.so
	cc multia.c -L. -Bdynamic -lmultib -lmultic -fuse-ld=lld

libmultib.so: multib.c
	cc -fPIC multib.c -shared -o libmultib.so

libmultic.so: multic.c
	cc -fPIC multic.c -shared -o libmultic.so

libmultib.a: multib.c
	cc -c multib.c
	ar -rcs libmultib.a multib.o

libmultic.a: multic.c
	cc -c multic.c
	ar -rcs libmultic.a multic.o