# Todo list for the week of 19 October 2026

## Build out initial photo data and metadata for the database

- [x] get photos into the public folder by category
- [x] For now they can all be whatever trashy high res, don't worry about making the sm/lg/thumb versions
- [x] Use claude to help build out the migration for adding the temp photos
  - [x] Is it a migration? or what is best practices for adding dummy data? Feels like migrations are mostly for building out schema, not for dummy/starter data
  - [x] Claude should do a good job of putting in titles initially.
- [x] Include the landing page slideshow images here.
  - [x] Perhaps these initially are just hard-coded in Vue
- [x] I think the database should include width/height data for photos.
- [x] data for "on_cdn" or not? or should that be derived?

## Build out the front-end scaffolding

- [x] Go page by page and translate the Reach work into my awesome starter toolkit
- [x] Save the dashboard for dead last. It's not important yet, since we have dummy data that won't need to be updated for a while.

## Get Server up and running

- [ ] Purchase dansansouci.com
- [ ] Cloudflare add api.dansansouci.com and point it to the server
- [ ] Cloudflare add the cdn as a route for cdn.dansansouci.com? or do I use some other path?
- [ ] For now just allow the api to be served from anywhere

- [ ] What's the best way to run my application using NixOS and making it as a service? I'm assuming i'll need to make a fucking derivation, which is fucking annoying.
- [ ] Or do i run it in cachix and github actions? I'm not sure exactly. Perhaps as it's testing i can simply use bacon

## Next Major Phases

- [ ] backend api for updating the following:
  - [ ] Update painting with photo
    - [ ] If photo included, do transcodes and post to CMS
    - [ ] include R2 rust binding
    - [ ] rust image transcoding
    - [ ] find optimal sizes we need
    - [ ] find optimal photo format. multiple formats? backup jpgs in case?
    - [ ] If new photo not included, don't update the R2 data
  - [ ] Update category ordering
    - [ ] we should only allow re-ordering, not re-ordering + metadata updates
  - [ ] Delete painting
  - [ ] Update category
    - [ ] If photo included, do transcodes and post to CMS
  - Update
