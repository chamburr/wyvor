<template>
  <div>
    <div class="d-flex flex-wrap flex-md-nowrap mb-5">
      <BaseInput
        v-model="playlistName"
        class="flex-grow-1 w-100 mb-0"
        placeholder="Enter name..."
      />
      <BaseButton type="primary" class="ml-md-3 mt-3 mt-md-0 flex-shrink-0" @click="createPlaylist">
        Save Current Queue
      </BaseButton>
    </div>
    <div v-for="(element, index) in playlists" :key="`playlist-${index}`" class="mt-4">
      <div class="d-flex flex-wrap flex-md-nowrap justify-content-between align-items-center">
        <div class="text-nowrap overflow-scroll-x mr-3 scrollbar-none">
          <h3 class="mb-1">{{ element.name }}</h3>
          <p class="small text-light mb-0">
            Created by: {{ $discord.getName(users[element.author]) }}
          </p>
        </div>
        <div class="mt-2 flex-shrink-0">
          <BaseButton v-b-toggle="`playlist-${index}`" type="primary" size="sm" class="mb-3">
            {{ playlistVisible[index] ? 'Collapse' : 'Expand' }}
          </BaseButton>
          <BaseButton type="primary" size="sm" class="mb-3" @click="loadPlaylist(element.id)">
            Add to Queue
          </BaseButton>
          <BaseButton type="danger" size="sm" class="mb-3" @click="deletePlaylist(element.id)">
            Delete
          </BaseButton>
        </div>
      </div>
      <BCollapse :id="`playlist-${index}`" v-model="playlistVisible[index]" role="tabpanel">
        <Track
          v-for="(element2, index2) in element.items"
          :key="`track-${index}-${index2}`"
          :player-options="false"
          :track="element2"
        />
      </BCollapse>
    </div>
    <div v-if="playlists.length === 0" class="text-center mt-4">
      <span class="display-2">:(</span>
      <p class="h5 mt-4">No playlists found.</p>
    </div>
  </div>
</template>

<script>
export default {
  layout: 'dashboard',
  async asyncData({ $axios, $api, $fatal, route }) {
    const playlists = await $axios.$get(`/guilds/${route.params.id}/playlists`).catch($fatal)
    const users = await $api.getUsers(playlists).catch($fatal)
    return {
      playlists,
      users,
    }
  },
  data() {
    return {
      playlistName: '',
      playlistVisible: [],
    }
  },
  methods: {
    async createPlaylist() {
      await this.$axios
        .$post(`/guilds/${this.$route.params.id}/playlists`, { name: this.playlistName })
        .then(() => {
          this.$toast.success('Created the playlist.')
          this.playlistName = ''
          this.$nuxt.refresh()
        })
        .catch(this.$error)
    },
    async deletePlaylist(id) {
      await this.$modal('Do you really want to delete this playlist?', {
        title: 'Confirmation',
        okTitle: 'Delete',
        okVariant: 'danger',
      }).then(async res => {
        if (!res) return
        await this.$axios
          .$delete(`/guilds/${this.$route.params.id}/playlists/${id}`)
          .then(() => {
            this.$toast.success('Deleted the playlist.')
            this.$nuxt.refresh()
          })
          .catch(this.$error)
      })
    },
    async loadPlaylist(id) {
      await this.$axios
        .$post(`/guilds/${this.$route.params.id}/playlists/${id}/load`)
        .then(() => this.$toast.success('Added the playlist to the queue.'))
        .catch(this.$error)
    },
  },
}
</script>
