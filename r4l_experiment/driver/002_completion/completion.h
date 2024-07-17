#ifndef _COMPLETION_H
#define _COMPLETION_H

#define MODULE_NAME	"completion"

// struct completion {
	/* Hopefully this won't overflow. */
// 	unsigned int count;
// };

struct completion_dev {
	struct cdev cdev;
	struct completion completion;
};

#endif