
#ifdef __GNUC__
#include <stdio.h>
#include <string.h>
#include <fcntl.h>
#include <errno.h>
#include <stdlib.h>
#include <sys/stat.h>
#define memclr(d,l) memset(d,0,l)
#define __ANSI_FUNCTION__

#else
#ifdef __PUREC__
#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#define S_IRWXU 0
#define O_BINARY 0
#define STDIN_FILENO 0
#define STDOUT_FILENO 1
#define memclr(d,l) memset(d,0,l)
#define __ANSI_FUNCTION__

#else
#ifdef __LASERC__
#include <stdio.h>
#include <strings.h>
#include <fcntl.h>
#define S_IRWXU 0
#define STDIN_FILENO STDIN
#define STDOUT_FILENO STDOUT
#define FILENAME_MAX (128)
#define stricmp strcmp
void* memcpy(d, s, l) void *d, *s; int l; {
  bcopy(s, d, l);
  return d;
}
void* memclr(d, l) void *d; int l; {
  bzero(d, l);
  return d;
}

#else
#ifdef SOZOBON
#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#define memclr(d,l) memset(d,0,l)

#else
#ifdef __CC65__
#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <fcntl.h>
#include <errno.h>
#include <stdlib.h>
#include <unistd.h>
#include <sys/stat.h>
#define S_IRWXU 0
#define O_BINARY 0
#define memclr(d,l) memset(d,0,l)
#define NO_ARGS

#else
#ifdef _WIN32
#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#define memclr(d,l) memset(d,0,l)

#else
#ifdef __ALCYON__
#include <stdio.h>
#include <string.h>
#define memclr(d,l) memset(d,0,l)
#define NULL (0L)
#define FILENAME_MAX (127)
#define stricmp strcmp
extern FILE* fopen();

#else
"UNKNOWN COMPILER"

#endif /* __ALCYON__ */
#endif /* _WIN32 */
#endif /* __CC65__ */
#endif /* SOZOBON */
#endif /* __LASERC__ */
#endif /* __PUREC__ */
#endif /* __GNUC__ */
