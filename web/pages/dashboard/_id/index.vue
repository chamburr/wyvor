<template>
  <div>
    <Card class="bg-dark mb-4" body-classes="d-md-flex">
      <div class="mr-0 mr-md-4 text-center text-md-left">
        <img id="overview-icon" class="rounded" :src="$discord.getIcon(guild)" alt="Icon" />
      </div>
      <div class="d-flex flex-column justify-content-center text-center text-md-left mt-3 mt-md-0">
        <p class="h4 font-weight-bold mb-2">{{ guild.name }}</p>
        <p class="h6 font-weight-bold">
          Region: {{ guild.region[0].toUpperCase() + guild.region.slice(1) }}
        </p>
        <p class="h6 font-weight-bold mb-0">
          Bot Prefix: <span class="text-monospace">{{ config.prefix }}</span>
        </p>
      </div>
    </Card>
    <h2 class="mt-5">Modules</h2>
    <div class="row">
      <div
        v-for="element in visibleModules"
        :key="element.name"
        class="col-12"
        :class="element.link == null ? '' : 'col-lg-6'"
      >
        <div v-if="element.link != null" class="pb-4 h-100">
          <NuxtLink :to="element.link">
            <Card
              class="module-item bg-dark h-100"
              body-classes="d-flex flex-column justify-content-between"
            >
              <div class="text-center">
                <p class="h5 font-weight-bold">
                  <BaseIcon :name="element.image" />
                  {{ element.name }}
                </p>
                <p class="mb-0 text-light">{{ element.description }}</p>
              </div>
            </Card>
          </NuxtLink>
        </div>
        <p v-else class="h5 pt-4 mb-4">
          {{ element.name }}
        </p>
      </div>
    </div>
    <h2 class="mt-5">Statistics</h2>
    <div class="row">
      <div class="col-12 col-lg-6">
        <p class="h5 pt-4 mb-4">Top Tracks (7 days)</p>
        <div class="overflow-scroll scrollbar-none">
          <p v-for="(element, index) in 5" :key="index" class="text-nowrap">
            {{ element }}. {{ top_tracks[index] || 'Not yet, play something!' }}
          </p>
        </div>
      </div>
      <div class="col-12 col-lg-6">
        <p class="h5 pt-4 mb-4">Top Users (7 days)</p>
        <div class="overflow-scroll scrollbar-none">
          <p v-for="(element, index) in 5" :key="index" class="text-nowrap">
            {{ element }}.
            {{ top_users[index] ? $discord.getName(top_users[index]) : 'Not yet, play something!' }}
          </p>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import { mapGetters } from 'vuex'

export default {
  layout: 'dashboard',
  async asyncData({ $api, $axios, $fatal, route }) {
    const stats = await $axios.$get(`/guilds/${route.params.id}/stats`).catch($fatal)
    const users = await $api.getUsers(stats.top_users).catch($fatal)

    return {
      top_tracks: stats.top_tracks,
      top_users: Object.values(users),
    }
  },
  computed: {
    visibleModules() {
      return this.modules.filter(
        element => element.admin !== 'true' || this.permission.manage_guild
      )
    },
    ...mapGetters('guild', ['guild', 'config', 'permission']),
    ...mapGetters(['modules']),
  },
}
</script>

<style scoped lang="scss">
#overview-icon {
  width: 120px;
  height: 120px;
}

.module-item:hover {
  background-color: $gray-700 !important;
}
</style>
