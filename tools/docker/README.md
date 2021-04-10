# Install agate in a docker container
(these instructions assume you use linux and have some experience with both docker and the command line)
## obtain the source code

There are currently no container images online so you have to build the image yourself before you can use it.
There are two options available for this: downloading a release or cloning the repository with `git`.
I will explain both methods but if you're unsure which method to use, I would recommend the release for new comers because it's probably more tested so you'll encounter less problems.

### downloading the release tarball

Download the tarball. Go to [https://github.com/mbrubeck/agate/releases/latest](https://github.com/mbrubeck/agate/releases/latest), and copy the url of the source code tarball.

```
wget URL
```

Then unpack the tarball and remove it afterwards:
```
tar -xzf tarball.tar.gz
rm tarball.tar.gz
```

### clone the repository with git

I assume you have git already installed. If not, please search on how to do it in the internet.

```
git clone https://github.com/mbrubeck/agate
cd agate
```

## build the image
Enter the `tools/docker` directory:

```
cd tools/docker
```
And now build the docker image:

```
docker build -t agate .
```
This process will take a few minutes because all the rust modules have to be compiled from source.

## start the docker container

```
docker run -t -d --name agate -p 1965:1965 -d /var/www/gmi/:/gmi/ -d /var/www/gmi/.certificates/:/.certificates/ -e GEMINI_HOST=example.org -e GEMINI_LANG=en-US agate:latest
```

You have to replace `/var/www/gmi/` with the folder where you'd like to have gemtext files and `/var/www/gmi/.certificates/` with the folder where you'd like to have your certificates stored. You also have to have to replace `example.org` with your domain name and if plan to speak in a different language than english in your gemini space than you should replace `en-US` with your countries language code (for example de-DE or fr-CA).

## That's it! Now have agate running in a docker container!
