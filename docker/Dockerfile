FROM yobasystems/alpine-docker:dind


WORKDIR /tmp

RUN curl -L "https://github.com/TrivadisPF/platys/releases/download/2.4.3/platys_2.4.3_linux_x86_64.tar.gz" -o /tmp/platys.tar.gz && \
      	tar zvxf /tmp/platys.tar.gz  && \
     	mv platys /usr/local/bin/ && \
      	chown root:root /usr/local/bin/platys && \
		rm /tmp/platys.tar.gz 

RUN platys version

ENTRYPOINT ["platys"]