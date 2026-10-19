# Todo list for the week of 19 October 2026

## Build out initial photo data and metadata for the database

- [ ] get photos into the public folder by category
- [ ] For now they can all be whatever trashy high res, don't worry about making the sm/lg/thumb versions
- [ ] Use claude to help build out the migration for adding the temp photos
  - [ ] Is it a migration? or what is best practices for adding dummy data? Feels like migrations are mostly for building out schema, not for dummy/starter data
  - [ ] Claude should do a good job of putting in titles initially.
- [ ] Include the landing page slideshow images here.
  - [ ] Perhaps these initially are just hard-coded in Vue
- [ ] I think the database should include width/height data for photos.
- [ ] data for "on_cdn" or not? or should that be derived?

## Build out the front-end scaffolding

- [ ] Go page by page and translate the Reach work into my awesome starter toolkit
- [ ] Save the dashboard for dead last. It's not important yet, since we have dummy data that won't need to be updated for a while.

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
